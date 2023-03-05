use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
enum RESP {
    SimpleString(String),
    Error(String),
    Integer(i32),
    BulkString(String),
    Array(Vec<RESP>),
}

impl RESP {
    fn encode(&self) -> String {
        match self {
            RESP::SimpleString(s) => format!("+{}\r\n", s),
            RESP::Error(e) => format!("-{}\r\n", e),
            RESP::Integer(n) => format!(":{}\r\n", n),
            RESP::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RESP::Array(arr) => {
                let mut s = format!("*{}\r\n", arr.len());
                for a in arr {
                    s.push_str(&a.encode());
                }
                s
            }
        }
    }
}

fn decode(text: &str) -> (RESP, usize) {
    // TODO: change to Result<Resp>
    assert!(text.len() > 0);
    println!("processing: {}", text);
    let end = text.find("\r\n").unwrap();
    match &text.chars().nth(0) {
        Some(c) => match c {
            '+' => (RESP::SimpleString(text[1..end].to_string()), end + 2),
            '-' => (RESP::Error(text[1..end].to_string()), end + 2),
            ':' => (RESP::Integer(text[1..end].parse().unwrap()), end + 2),
            '$' => {
                let len: usize = text[1..end].parse().unwrap();
                (
                    RESP::BulkString(text[(end + 2)..(end + 2 + len)].to_string()),
                    end + 2 + len + 2,
                )
            }
            '*' => {
                let len: usize = text[1..end].parse().unwrap();
                let mut res = Vec::new();
                let mut ptr = end + 2;
                for _ in 0..len {
                    let (r, c) = decode(&text[ptr..text.len()]);
                    res.push(r);
                    ptr = ptr + c;
                }
                (RESP::Array(res), ptr)
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

fn process_text(text: &str) -> RESP {
    match text {
        "PING" | "ping" => RESP::SimpleString("PONG".to_string()),
        "COMMAND" | "command" => RESP::Array(vec![RESP::SimpleString("PING".to_string())]),
        s => panic!("Unknown query: {}", s),
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
        _ => panic!(),
    }
}

fn process_stream(stream: &mut TcpStream) -> std::io::Result<()> {
    loop {
        let mut input = Vec::new();
        loop {
            let mut tmp_input = [0u8; 1024];
            let count = stream.read(&mut tmp_input).unwrap_or_default();
            if count == 0 {
                break;
            }
            input.append(&mut tmp_input[0..count].to_vec());
            if input.len() >= 2 && input.ends_with(&[b'\r', b'\n']) {
                break;
            }
        }

        if input.len() == 0 {
            return stream.shutdown(std::net::Shutdown::Both);
        }

        let raw_input = String::from_utf8(input).unwrap();
        let queries = decode(&raw_input).0;
        // println!("{}", raw_input);
        // println!("{:?}", queries);

        let response_resp = process(&queries);
        let response_text = response_resp.encode();
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
        match stream.map(|mut stream| process_stream(&mut stream)) {
            Ok(_) => (),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
