#[derive(Debug)]
pub enum RESP {
    SimpleString(String),
    Error(String),
    Integer(i32),
    BulkString(String),
    Array(Vec<RESP>),
}

impl RESP {
    pub fn encode(&self) -> String {
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

    // TODO: change to Result<Resp>
    pub fn decode(text: &str) -> RESP {
        decode_internal(text).0
    }
}

fn decode_internal(text: &str) -> (RESP, usize) {
    assert!(text.len() > 0);
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
                    let (r, c) = decode_internal(&text[ptr..text.len()]);
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
