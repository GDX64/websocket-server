use anyhow::Result;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
    join, select,
};

use crate::websocket::OpCode;
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
        let msg = socket.read_frame().await?;
        match msg.opcode {
            OpCode::Ping => {
                println!("got ping");
                socket.pong().await?;
            }
            OpCode::Text | OpCode::Binary => {
                println!("{}", msg.text());
                socket.answer_string("Hello").await?;
            }
            OpCode::Pong => {
                println!("got pong");
            }
            OpCode::Close => {
                println!("got close");
            }
            _ => {
                println!("got something else");
            }
        }
    }
}
