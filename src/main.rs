mod redis;
use crate::redis::command::Command;
use crate::redis::resp::RESP;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

fn process(query: &RESP) -> RESP {
    match Command::from_resp(query) {
        Command::Ping => RESP::SimpleString("PONG".to_string()),
        Command::Commands => Command::supported_commands(),
        Command::Echo(s) => RESP::BulkString(s),
        Command::Unknown(_) => RESP::Error(format!("No such command: {:?}", query)),
    }
}

async fn process_socket(socket: &mut TcpStream) -> anyhow::Result<()> {
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
        let queries = RESP::from_str(&raw_input)?;
        // println!("{}", raw_input);
        // println!("{:?}", queries);

        let response_resp = process(&queries);
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

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = process_socket(&mut socket).await {
                println!("{}", e);
            };
        });
    }
}
