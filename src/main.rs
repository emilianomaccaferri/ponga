use axum::{extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::Response, routing::get, Router};
use futures::{SinkExt, StreamExt};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpSocket};


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
    
    tokio::spawn(async move {
        while let Some(item) = read_ws.next().await {
            match item {
                Ok(message) => {
                    match message {
                        Message::Binary(vec) => {
                            write_ssh.write_all(&vec).await
                                .expect("couldn't write to ssh server!");
                        }
                        _ => break,
                    }
                },
                Err(e) => println!("{}", e.to_string())
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let mut buf = [0u8; 1024];
            if let Ok(n) = read_ssh.read(&mut buf).await {
                if n == 0 {
                    write_ws.close().await
                        .expect("couldn't close ws connection");
                    break;
                }
                write_ws.send(
                    Message::Binary(Vec::from(&buf[0..n]))
                ).await
                .expect("couldnt write to ssh server");
            }
        }
    });
    
}
