use super::payload::Payload;

pub enum Message {
    Loaded(Box<Payload>),
    Failed(String, String),
}

impl Message {
    pub fn name(&self) -> String {
        match self {
            Message::Loaded(payload) => payload.video().ty().name(),
            Message::Failed(name, _) => name.clone(),
        }
    }
}
