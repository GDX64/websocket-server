use anyhow::Result;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
};
mod websocket;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        println!("Accepted connection");
        tokio::spawn(async move {
            match handle_websocket(socket).await {
                Ok(_) => println!("Socket closed"),
                Err(e) => println!("Connection error: {:?}", e),
            }
        });
    }
}

async fn handle_websocket(socket: tokio::net::TcpStream) -> Result<()> {
    //finished handshake
    let mut socket = websocket::Websocket::new(socket);
    socket.handshake().await?;
    loop {
        println!("=====client says =====");
        let msg = socket.read_frame().await?;
        println!("{msg:?}");
    }
}
