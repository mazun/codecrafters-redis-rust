mod redis;
use std::sync::Arc;

use crate::redis::command::Command;
use crate::redis::engine::RedisEngine;
use crate::redis::resp::RESP;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn process_socket(socket: &mut TcpStream, engine: Arc<RedisEngine>) -> anyhow::Result<()> {
    loop {
        let mut input = Vec::new();
        loop {
            let mut tmp_input = [0u8; 1024];
            let count = socket.read(&mut tmp_input).await?;
            if count == 0 {
                break;
            }
            input.append(&mut tmp_input[0..count].to_vec());
            if count < tmp_input.len() && input.len() >= 2 && input.ends_with(&[b'\r', b'\n']) {
                break;
            }
        }

        if input.len() == 0 {
            return Ok(());
        }

        let raw_input = String::from_utf8(input).unwrap();
        let query = RESP::from_str(&raw_input)?;
        // println!("{}", raw_input);
        // println!("{:?}", query);

        let response_resp = engine.process_command(Command::from_resp(&query));
        let response_text = response_resp.to_string();
        // println!("response_resp: {:?}", response_resp);
        // println!("response: {}", response_text);
        socket.write(response_text.as_bytes()).await?;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let engine = Arc::new(RedisEngine::new());

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let e = engine.clone();
        tokio::spawn(async move {
            if let Err(e) = process_socket(&mut socket, e).await {
                println!("{}", e);
            };
        });
    }
}
