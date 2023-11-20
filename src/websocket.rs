use std::{
    borrow::BorrowMut,
    io::{BufRead, Cursor, Read},
};

use anyhow::Result;
use bytes::{Buf, BytesMut};
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub struct Websocket {
    stream: tokio::net::TcpStream,
    buff: BytesMut,
}

impl Websocket {
    pub fn new(stream: tokio::net::TcpStream) -> Self {
        Self {
            stream,
            buff: BytesMut::with_capacity(1024),
        }
    }

    pub async fn handshake(&mut self) -> Result<()> {
        let n = self.stream.read_buf(&mut self.buff).await?;
        println!("read {} bytes", n);
        let msg = String::from_utf8_lossy(&self.buff);
        let result = handshake::handshake(&msg)?;
        self.stream.write_all(result.as_bytes()).await?;
        self.buff.clear();
        return Ok(());
    }

    pub async fn read_frame(&mut self) -> Result<String> {
        loop {
            match self.try_read_frame() {
                Some(msg) => {
                    return Ok(msg);
                }
                None => {
                    let n = self.stream.read_buf(&mut self.buff).await?;
                    if n == 0 {
                        return Err(anyhow::anyhow!("Socket closed by client"));
                    }
                }
            }
        }
    }

    pub fn try_read_frame(&mut self) -> Option<String> {
        if self.buff.len() < 2 {
            return None;
        }

        let data: &[u8] = &self.buff[..];
        let mut cursor = Cursor::new(data);
        let mut bytes = cursor.borrow_mut().bytes();

        let _first_byte = bytes.next()?.ok()?;
        let payload_byte = bytes.next()?.ok()?;
        let _mask_bit = (payload_byte & 0b1000_0000) >> 7;
        let payload_len = payload_byte & 0b0111_1111;
        let final_payload_len = if payload_len == 126 {
            todo!()
        } else if payload_len == 127 {
            todo!()
        } else {
            payload_len
        };
        let mask = [
            bytes.next()?.ok()?,
            bytes.next()?.ok()?,
            bytes.next()?.ok()?,
            bytes.next()?.ok()?,
        ];

        let cursor_pos = cursor.position() as usize;
        if self.buff.len() < cursor_pos + final_payload_len as usize {
            return None;
        }
        let final_pos = cursor_pos + final_payload_len as usize;
        let decoded_payload = data[cursor_pos..final_pos]
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ mask[i % 4])
            .collect::<Vec<u8>>();
        let res = String::from_utf8(decoded_payload).ok()?;
        self.buff.advance(final_pos);
        Some(res)
    }
}

mod handshake {
    use anyhow::Result;
    use base64::Engine;
    use sha1::{Digest, Sha1};
    use std::collections::HashMap;
    const MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    pub fn handshake(msg: &str) -> Result<String> {
        let headers = process_headers(&msg);
        let key = headers
            .get("Sec-WebSocket-Key")
            .ok_or(anyhow::anyhow!("No key"))?;
        let result = form_handshake_response(key);
        Ok(result)
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

    #[test]
    fn magic_answer() {
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let result = encode_key_anser(key);
        assert_eq!(result, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }
}
