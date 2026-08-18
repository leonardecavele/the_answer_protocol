#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub sender: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum ChatChannel {
    Global,
    Private(String),
    Room,
    Group,
}
