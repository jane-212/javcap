use nfo::Nfo;
use video::Video;

pub enum Message {
    Loaded(Box<Video>, Box<Nfo>),
    Failed(String, String),
}
