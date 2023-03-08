use crate::RESP;

#[derive(Debug)]
pub enum Command {
    Ping,
    Commands,
    Echo(String),
    Unknown(String),
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
            match op.as_str() {
                "PING" | "ping" => Some(Command::Ping),
                "COMMAND" | "command" => Some(Command::Commands),
                "ECHO" | "echo" => {
                    if len == 2 {
                        args[1].get_string().map(|s| Command::Echo(s.to_string()))
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
        let commands = vec!["COMMANDS", "PING", "ECHO"];
        RESP::Array(
            commands
                .into_iter()
                .map(|s| RESP::SimpleString(s.to_string()))
                .collect(),
        )
    }
}
