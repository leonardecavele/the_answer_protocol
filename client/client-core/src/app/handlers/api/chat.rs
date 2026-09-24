use crate::app::App;
use crate::notification::Notification;
use crate::states::game::{ChatChannel, ChatMessage, ChatSender, ChatState};

impl App {
    pub fn on_chat_received(&mut self, channel: ChatChannel, sender: String, content: String) {
        let is_me = Some(sender.clone()) == self.state.game.player.name;

        if !is_me {
            if let ChatChannel::Private(_) = channel {
                self.state.ui.notifications.push(Notification::info(format!(
                    "New private message from {}.",
                    sender
                )));
            }

            if !self.state.game.overlays.is_open::<ChatState>() {
                self.state.game.is_chat_unread = true;
            }
        }

        self.state
            .game
            .log_action(format!("{} ({}): {}", channel.prefix(), sender, content));

        let chat_message = if is_me {
            ChatMessage {
                channel,
                sender: ChatSender::Me,
                content,
            }
        } else {
            ChatMessage {
                channel,
                sender: ChatSender::Other(sender),
                content,
            }
        };

        self.state.game.chat_log.push(chat_message);
    }
}
