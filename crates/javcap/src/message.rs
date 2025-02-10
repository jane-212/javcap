use std::fmt::{self, Display};

use super::payload::Payload;

pub enum Message {
    Loaded(Box<Payload>),
    Failed(String, String),
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Loaded(payload) => write!(f, "{}", payload.video().ty()),
            Message::Failed(name, _) => write!(f, "{name}"),
        }
    }
}
