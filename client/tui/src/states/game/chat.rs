use std::ops::Deref;

const MAX_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub enum ChatChannel {
    Global,
    Private(String),
    Room,
    Group,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub sender: String,
    pub content: String,
}

pub struct ChatState(Vec<ChatMessage>);

impl ChatState {
    pub fn new() -> Self {
        ChatState(Vec::new())
    }

    pub fn push(&mut self, message: ChatMessage) {
        if self.0.len() == MAX_SIZE {
            self.0.remove(0);
        }
        self.0.push(message);
    }
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ChatState {
    type Target = [ChatMessage];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a ChatState {
    type Item = &'a ChatMessage;
    type IntoIter = std::slice::Iter<'a, ChatMessage>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<ChatMessage> for ChatState {
    fn from_iter<I: IntoIterator<Item = ChatMessage>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
