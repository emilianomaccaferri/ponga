use std::{env, io::Write};

use futures::SinkExt;
use futures_util::{future, pin_mut, StreamExt};
use tokio::{fs::read, io::{AsyncReadExt, AsyncWriteExt}};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, protocol::Message}};

// let mut stdin = tokio::io::stdin();
// tokio::io::stdout().write_all(&data).await.unwrap();

#[tokio::main]
async fn main(){
    let (ws_stream, _) = connect_async("ws://localhost:3000".to_string()).await.expect("couldn't connect");
    let (mut write, mut read) = ws_stream.split();

    let stdout_handle = tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(stuff) => {
                    // i have to investigate why tokio's stdout does not work as expected...
                    std::io::stdout().write_all(&stuff.into_data()).unwrap(); 
                    std::io::stdout().flush().unwrap();
                }
                Err(e) => {
                    println!("ws closed: {}", e.to_string());
                }
            }
        }
    });

    let stdin_handle = tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        loop {
            let mut buf = vec![0; 8093];
            if let Ok(bytes) = stdin.read(&mut buf).await {
                tokio::io::stdout().flush().await.expect("couldn't flush");
                let message = &buf.clone()[0..bytes];
                if let Err(e) = write.send(Message::binary(message)).await {
                    println!("couldnt send to ws stream: {}", e.to_string());
                }
            }
        }
    });

    pin_mut!(stdout_handle, stdin_handle);
    future::select(stdin_handle, stdout_handle).await;

}