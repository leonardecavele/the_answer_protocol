use crate::app::runtime::App;
use crate::states::Notification;
use crate::states::game::{ChatChannel, ChatMessage, ChatSender};

impl App {
    pub fn on_global_chat_sent(&mut self, message: String) {
        self.state.game.chat_log.push(ChatMessage {
            channel: ChatChannel::Global,
            sender: ChatSender::Me,
            content: message,
        });
    }

    pub fn on_room_chat_sent(&mut self, message: String) {
        self.state.game.chat_log.push(ChatMessage {
            channel: ChatChannel::Room,
            sender: ChatSender::Me,
            content: message,
        });
    }

    pub fn on_group_chat_sent(&mut self, message: String) {
        self.state.game.chat_log.push(ChatMessage {
            channel: ChatChannel::Group,
            sender: ChatSender::Me,
            content: message,
        });
    }

    pub fn on_private_chat_sent(&mut self, to: String, message: String) {
        self.state.game.chat_log.push(ChatMessage {
            sender: ChatSender::Me,
            channel: ChatChannel::Private(to),
            content: message,
        });
    }

    pub fn on_chat_received(&mut self, channel: ChatChannel, sender: String, content: String) {
        if self.state.game.player.name.as_deref() == Some(sender.as_str()) {
            return;
        }

        if let ChatChannel::Private(_) = channel {
            self.state.ui.notifications.push(Notification::info(format!(
                "New private message from {}.",
                sender
            )));
        }

        self.state.game.chat_log.push(ChatMessage {
            channel,
            sender: ChatSender::Other(sender),
            content,
        });
    }
}
