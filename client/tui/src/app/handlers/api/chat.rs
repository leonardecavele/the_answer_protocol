use crate::app::App;
use crate::states::game::{ChatChannel, ChatMessage};
use crate::states::ui::Notification;

impl App {
    pub(crate) fn on_global_chat_sent(&mut self, message: String) {
        self.state.game.chat_log.push(ChatMessage {
            channel: ChatChannel::Global,
            sender: "You".to_string(),
            content: message,
        });
    }

    pub(crate) fn on_private_chat_sent(&mut self, to: String, message: String) {
        self.state.game.chat_log.push(ChatMessage {
            sender: format!("(You) to {}", to),
            channel: ChatChannel::Private(to),
            content: message,
        });
    }

    pub(crate) fn on_chat_received(
        &mut self,
        channel: ChatChannel,
        sender: String,
        content: String,
    ) {
        if let ChatChannel::Private(_) = channel {
            self.state.ui.notifications.push(Notification::info(format!(
                "New private message from {}.",
                sender
            )));
        }

        self.state.game.chat_log.push(ChatMessage {
            channel,
            sender,
            content,
        });
    }
}
