use crate::{Command, RESP};

#[derive(Clone, Copy, Debug)]
pub struct RedisEngine {}

impl RedisEngine {
    pub fn process_command(&mut self, command: Command) -> RESP {
        match command {
            Command::Ping => RESP::SimpleString("PONG".to_string()),
            Command::Commands => Command::supported_commands(),
            Command::Echo(s) => RESP::BulkString(s),
            Command::Unknown(q) => RESP::Error(format!("No such command: {:?}", q)),
        }
    }
}
