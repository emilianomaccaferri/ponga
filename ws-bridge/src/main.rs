mod channels;

use std::{env, io::Write};

use channels::BridgeCommand;
use futures::SinkExt;
use futures_util::{future, StreamExt};
use tokio::{fs::read, io::{AsyncReadExt, AsyncWriteExt}};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, protocol::Message}};
use tokio_util::sync::CancellationToken;

// let mut stdin = tokio::io::stdin();
// tokio::io::stdout().write_all(&data).await.unwrap();

#[tokio::main]
async fn main(){
    let canc_token = CancellationToken::new();

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
        tokio::select! {
            _ = 
        }
    });

    tokio::select! {
        _ = stdin_handle => {
            println!("stopped reading stdin");
        },
        _ = stdout_handle => {
            println!("stopped writing to stdout");
        },
        _ = tokio::signal::ctrl_c() => {
        },
    }
}