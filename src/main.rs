use anyhow::Result;
use std::time::SystemTime;
use tokio::{self, time::timeout};

use crate::websocket::OpCode;
mod websocket;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1234").await.unwrap();
    let server_start_time = SystemTime::now();
    let mut count = 0;
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let this_connection = count;
        count += 1;
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(server_start_time)
            .expect("Time went backwards");
        println!(
            "{}: new connection {}",
            since_the_epoch.as_millis(),
            this_connection
        );
        tokio::spawn(async move {
            match handle_websocket(socket).await {
                Ok(_) => println!("Socket closed"),
                Err(e) => println!("Connection error: {:?}", e),
            }
        });
        println!("spawned new task for connection {}", this_connection);
    }
}

async fn handle_websocket(socket: tokio::net::TcpStream) -> Result<()> {
    //finished handshake
    let mut socket = websocket::Websocket::new(socket);
    socket.handshake().await?;
    loop {
        let msg = socket.read_frame().await?;
        // let msg = timeout(tokio::time::Duration::from_secs(2), msg).await;
        // let msg = if let Ok(msg) = msg {
        //     msg?
        // } else {
        //     // println!("make ping");
        //     socket.ping().await?;
        //     continue;
        // };
        match msg.opcode {
            OpCode::Ping => {
                // println!("got ping");
                socket.pong().await?;
            }
            OpCode::Text | OpCode::Binary => {
                // println!("{}", msg.text());
                socket.answer_string(msg.text()).await?;
            }
            OpCode::Pong => {
                // println!("got pong");
            }
            OpCode::Close => {
                // println!("got close");
            }
            _ => {
                // println!("got something else");
            }
        }
    }
}
