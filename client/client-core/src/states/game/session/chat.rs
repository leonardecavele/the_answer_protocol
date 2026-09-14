#[derive(Debug, Clone)]
pub enum ChatSender {
    Me,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub sender: ChatSender,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum ChatChannel {
    Global,
    Private(String),
    Room,
    Group,
}

impl ChatChannel {
    pub fn prefix(&self) -> &'static str {
        match self {
            ChatChannel::Global => "[GLOBAL]",
            ChatChannel::Group => "[GROUP]",
            ChatChannel::Room => "[ROOM]",
            ChatChannel::Private(_) => "[PRIVATE]",
        }
    }
}
