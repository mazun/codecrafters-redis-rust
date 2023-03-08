use std::{collections::HashMap, sync::Mutex};

use bytes::Bytes;

use crate::{Command, RESP};

#[derive(Debug)]
pub struct RedisEngine {
    db: Mutex<HashMap<String, Bytes>>,
}

impl RedisEngine {
    pub fn new() -> RedisEngine {
        RedisEngine {
            db: Mutex::new(HashMap::new()),
        }
    }

    pub fn process_command(&self, command: Command) -> RESP {
        match command {
            Command::Ping => RESP::SimpleString("PONG".to_string()),
            Command::Commands => Command::supported_commands(),
            Command::Echo(s) => RESP::BulkString(s),
            Command::Unknown(q) => RESP::Error(format!("No such command: {:?}", q)),
            Command::Set { key, value } => {
                let mut db = match self.db.lock() {
                    Ok(db) => db,
                    Err(e) => return RESP::Error(e.to_string()),
                };
                db.insert(key, Bytes::from(value));
                RESP::SimpleString("OK".to_string())
            }
            Command::Get(key) => {
                let db = match self.db.lock() {
                    Ok(db) => db,
                    Err(e) => return RESP::Error(e.to_string()),
                };
                if let Some(value) = db.get(&key) {
                    RESP::BulkString(String::from_utf8(value.to_vec()).unwrap())
                } else {
                    RESP::Nil
                }
            }
        }
    }
}
