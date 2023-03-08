use anyhow::{anyhow, Result};

#[derive(Debug)]
pub enum RESP {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RESP>),
}

impl RESP {
    pub fn to_string(&self) -> String {
        match self {
            RESP::SimpleString(s) => format!("+{}\r\n", s),
            RESP::Error(e) => format!("-{}\r\n", e),
            RESP::Integer(n) => format!(":{}\r\n", n),
            RESP::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RESP::Array(arr) => {
                format!(
                    "*{}\r\n{}",
                    arr.len(),
                    arr.into_iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
        }
    }

    pub fn get_string(&self) -> Option<&String> {
        match self {
            RESP::SimpleString(s) => Some(s),
            RESP::BulkString(s) => Some(s),
            _ => None,
        }
    }

    pub fn from_str(text: &str) -> Result<RESP> {
        let (r, used_bytes) = decode_internal(text)?;
        if used_bytes == text.len() {
            Ok(r)
        } else {
            Err(anyhow!(
                "Failed to parse the entire input as a RESP: {}",
                text
            ))
        }
    }
}

fn decode_internal(text: &str) -> Result<(RESP, usize)> {
    if text.len() == 0 {
        return Err(anyhow!("Failed to parse an empty input as a RESP"));
    }
    let end = text.find("\r\n").unwrap();
    match &text.chars().nth(0) {
        Some(c) => match c {
            '+' => Ok((RESP::SimpleString(text[1..end].to_string()), end + 2)),
            '-' => Ok((RESP::Error(text[1..end].to_string()), end + 2)),
            ':' => Ok((RESP::Integer(text[1..end].parse()?), end + 2)),
            '$' => {
                let len: usize = text[1..end].parse()?;
                Ok((
                    RESP::BulkString(text[(end + 2)..(end + 2 + len)].to_string()),
                    end + 2 + len + 2,
                ))
            }
            '*' => {
                let len: usize = text[1..end].parse()?;
                let mut res = Vec::new();
                let mut ptr = end + 2;
                for _ in 0..len {
                    let (r, c) = decode_internal(&text[ptr..])?;
                    res.push(r);
                    ptr = ptr + c;
                }
                Ok((RESP::Array(res), ptr))
            }
            _ => Err(anyhow!("Failed to parse RESP: {}", text)),
        },
        _ => Err(anyhow!("Failed to parse RESP: {}", text)),
    }
}
