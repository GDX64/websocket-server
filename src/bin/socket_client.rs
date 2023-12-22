use anyhow::Result;
use tokio::{self};
use websockets::Websocket;

#[tokio::main]
async fn main() {
    let socket = tokio::net::TcpStream::connect("127.0.0.1:1234")
        .await
        .unwrap();
    if let Err(e) = handle_websocket(socket).await {
        println!("Connection error: {:?}", e);
    }
}

async fn handle_websocket(socket: tokio::net::TcpStream) -> Result<()> {
    let mut socket = Websocket::new(socket);
    socket.client_handshake().await?;
    let frame = socket.read_frame().await?;
    println!("this is the text ans{}", frame.text());
    socket.answer_string("hi there").await?;
    loop {
        let frame = socket.read_frame().await?;
        println!("this is the text ans{}", frame.text());
    }
}
