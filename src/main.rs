use std::sync::{Arc, Mutex};
use axum::{extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::Response, routing::get, Router};
use futures::{SinkExt, StreamExt};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpSocket, select};


#[tokio::main]
async fn main() {
    let app: Router = Router::new()
        .route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|sck| handle(sck))
}

async fn handle(socket: WebSocket) {
    let addr = "127.0.0.1:22".parse().unwrap();

    let ssh_socket = TcpSocket::new_v4().unwrap();
    let stream = ssh_socket.connect(addr).await.unwrap();
    let (mut read_ssh, mut write_ssh) = stream.into_split();
    let (mut write_ws, mut read_ws) = socket.split();

    // from websocket to ssh
    tokio::spawn(async move {

        while let Some(item) = read_ws.next().await {
            match item {
                Ok(message) => {
                    match message {
                        Message::Binary(vec) => {
                            println!("received: {}", String::from_utf8_lossy(&vec));
                            if let Err(e) = write_ssh.write(&vec).await {
                                println!("something went wrong while writing to ssh: {}", e.to_string());
                            }
                        },
                        _ => break
                    }
                },
                Err(e) => {
                    println!("error on write: {}", e.to_string());
                }
            }
        }

        println!("stopped writing to ssh");
        
    });

    // from ssh to websocket
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match read_ssh.read(&mut buffer).await {
                Ok(n) => {
                    if n > 0 {
                        if let Err(e) = write_ws.send(Message::Binary(buffer[..n].to_vec())).await {
                            println!("couldn't send to websocket: {}", e);
                            break;
                        }
                    } else {
                        println!("ssh closed connection");
                        break;
                    }
                }
                Err(e) => {
                    println!("error on read: {}", e.to_string());
                    break;
                }
            }
        }
        println!("stopped reading from ssh");

    });
    
}
