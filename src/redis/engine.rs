use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use bytes::Bytes;

use crate::{Command, RESP};

#[derive(Debug)]
struct DbData {
    data: Bytes,
    expire: Option<Instant>,
}

#[derive(Debug)]
pub struct RedisEngine {
    db: Mutex<HashMap<String, DbData>>,
}

impl RedisEngine {
    pub fn new() -> RedisEngine {
        RedisEngine {
            db: Mutex::new(HashMap::new()),
        }
    }

    pub fn process_command(&self, command: Command) -> RESP {
        // println!("command: {:?}", command);
        match command {
            Command::Ping => RESP::SimpleString("PONG".to_string()),
            Command::Commands => Command::supported_commands(),
            Command::Echo(s) => RESP::BulkString(s),
            Command::Unknown(q) => RESP::Error(format!("No such command: {:?}", q)),
            Command::Set {
                key,
                value,
                set_option,
            } => {
                let mut db = match self.db.lock() {
                    Ok(db) => db,
                    Err(e) => return RESP::Error(e.to_string()),
                };
                let mut expire = None;
                if let Some(expire_milliseconds) = set_option.expire {
                    let now = Instant::now();
                    expire = Some(
                        now + Duration::new(
                            expire_milliseconds / 1000,
                            ((expire_milliseconds % 1000) * 1000000) as u32,
                        ),
                    );
                }

                db.insert(
                    key,
                    DbData {
                        data: Bytes::from(value),
                        expire: expire,
                    },
                );
                RESP::SimpleString("OK".to_string())
            }
            Command::Get(key) => {
                // TODO: Make this mutable only if the value is expired
                let mut db = match self.db.lock() {
                    Ok(db) => db,
                    Err(e) => return RESP::Error(e.to_string()),
                };
                if let Some(value) = db.get(&key) {
                    if let Some(expire) = value.expire {
                        let current = Instant::now();
                        if current > expire {
                            // println!("{}: expired ({:?} > {:?})", key, current, expire);
                            db.remove(&key);
                            return RESP::Nil;
                        }
                    }
                    RESP::BulkString(String::from_utf8(value.data.to_vec()).unwrap())
                } else {
                    RESP::Nil
                }
            }
        }
    }
}
