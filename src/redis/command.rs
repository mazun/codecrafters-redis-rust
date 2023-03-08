use crate::RESP;

#[derive(Debug)]
pub enum Command {
    Ping,
    Commands,
    Echo(String),
    Unknown(String),
    Set { key: String, value: String },
    Get(String),
}

impl Command {
    fn from_resp_internal(resp: &RESP) -> Option<Command> {
        if let RESP::Array(args) = &resp {
            let len = args.len();
            if len == 0 {
                return None;
            }
            let op = match &args[0] {
                RESP::SimpleString(s) => s,
                RESP::BulkString(s) => s,
                _ => return None,
            };
            match op.to_lowercase().as_str() {
                "ping" => Some(Command::Ping),
                "command" => Some(Command::Commands),
                "echo" => {
                    if len == 2 {
                        args[1].get_string().map(|s| Command::Echo(s.to_string()))
                    } else {
                        None
                    }
                }
                "set" => {
                    if len >= 3 {
                        let key = args[1].get_string()?;
                        let value = args[2].get_string()?;
                        Some(Command::Set {
                            key: key.to_string(),
                            value: value.to_string(),
                        })
                    } else {
                        None
                    }
                }
                "get" => {
                    if len == 2 {
                        args[1].get_string().map(|s| Command::Get(s.to_string()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn from_resp(resp: &RESP) -> Command {
        Command::from_resp_internal(&resp).unwrap_or(Command::Unknown(resp.to_string()))
    }

    // pub fn from_resps<I: Iterator<Item = RESP>>(mut resps: I) -> Vec<Command> {
    //     resps.map(|resp| Command::from_resp(&resp)).collect()
    // }

    pub fn supported_commands() -> RESP {
        let commands = vec!["COMMANDS", "PING", "ECHO", "SET", "GET"];
        RESP::Array(
            commands
                .into_iter()
                .map(|s| RESP::SimpleString(s.to_string()))
                .collect(),
        )
    }
}
