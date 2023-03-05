mod redis;
use crate::redis::resp::RESP;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn process_text(text: &str) -> RESP {
    match text {
        "PING" | "ping" => RESP::SimpleString("PONG".to_string()),
        "COMMAND" | "command" => RESP::Array(vec![RESP::SimpleString("PING".to_string())]),
        _ => RESP::Error(format!("No such command: {}", text)),
    }
}

fn process(query: &RESP) -> RESP {
    match &query {
        RESP::SimpleString(s) => process_text(s),
        RESP::BulkString(s) => process_text(s),
        RESP::Array(arr) => match arr.len() {
            1 => process(&arr[0]),
            _ => RESP::Array(arr.into_iter().map(process).collect()),
        },
        _ => RESP::Error(format!("Invalid query: {:?}", query)),
    }
}

fn process_stream(stream: &mut TcpStream) -> anyhow::Result<()> {
    loop {
        let mut input = Vec::new();
        loop {
            let mut tmp_input = [0u8; 1024];
            let count = stream.read(&mut tmp_input).unwrap_or_default();
            if count == 0 {
                break;
            }
            input.append(&mut tmp_input[0..count].to_vec());
            if count < tmp_input.len() && input.len() >= 2 && input.ends_with(&[b'\r', b'\n']) {
                break;
            }
        }

        if input.len() == 0 {
            stream.shutdown(std::net::Shutdown::Both)?;
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
        stream.write_all(response_text.as_bytes())?;
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        if let Err(e) = stream.map(|mut stream| process_stream(&mut stream)) {
            println!("error: {}", e);
        }
    }
}
