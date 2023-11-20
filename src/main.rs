use anyhow::Result;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
};

const MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

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

async fn handle_websocket(mut socket: tokio::net::TcpStream) -> Result<()> {
    let mut buf = [0; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        let msg = String::from_utf8_lossy(&buf[..n]);
        println!("{}", msg);
        let headers = process_headers(&msg);
        let key = headers
            .get("Sec-WebSocket-Key")
            .ok_or(anyhow::anyhow!("No key"))?;
        let result = form_handshake_response(key);
        println!("{:?}", result);
        socket.write_all(result.as_bytes()).await?;
        let n = socket.read(&mut buf).await?;
        let msg = &buf[..n];
        println!("=====client says again =====");
        println!("{msg:?}");
        read_data_frame(msg);
    }
}

fn process_headers(headers: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in headers.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(": ") {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

fn encode_base64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn encode_key_anser(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key);
    hasher.update(MAGIC_STRING);
    let result: Vec<u8> = hasher.finalize().to_vec();
    let result = encode_base64(&result);
    result
}

fn form_handshake_response(key: &str) -> String {
    let result = encode_key_anser(key);
    format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Connection: Upgrade\r\n\
        Upgrade: websocket\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        result
    )
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn magic_answer() {
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let result = encode_key_anser(key);
        assert_eq!(result, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }
}

fn read_data_frame(data: &[u8]) {
    let _first_byte = data[0];
    let payload_byte = data[1];
    let mask_bit = (payload_byte & 0b1000_0000) >> 7;
    let payload_len = payload_byte & 0b0111_1111;
    println!("mask: {mask_bit}, payload_len: {payload_len}");
    let mask: [u8; 4] = data[2..6].try_into().unwrap();
    let payload = &data[6..];
    let decoded_payload = payload
        .iter()
        .enumerate()
        .map(|(i, byte)| byte ^ mask[i % 4])
        .collect::<Vec<u8>>();
    println!("{:?}", String::from_utf8_lossy(&decoded_payload))
}
