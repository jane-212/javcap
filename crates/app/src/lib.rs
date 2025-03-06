use std::sync::LazyLock;

pub const NAME: &str = "javcap";
pub const VERSION: &str = env!("VERSION");
pub const HASH: &str = env!("HASH");
pub static LINE_LENGTH: LazyLock<usize> = LazyLock::new(|| {
    terminal_size::terminal_size()
        .map(|(width, _)| width.0 as usize)
        .unwrap_or(40)
});
pub const USER_AGENT: &str = concat!("javcap", "/", env!("VERSION"));
