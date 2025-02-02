use video::Video;

pub enum Message {
    Loaded(Box<Video>),
    Failed(String, String),
}
