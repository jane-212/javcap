use video::Video;

pub enum Message {
    Load(Box<Video>),
    Failed(String, String),
}
