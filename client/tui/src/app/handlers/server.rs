use crate::app::App;

use crate::states::game::{ChatChannel, ChatMessage};
use api_client::client::event::{GroupEvent, RoomEvent, ServerEvent};

impl App {
    pub(crate) fn handle_server_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Connect(name) => {
                self.state.game.online_players_count += 1;
                self.state
                    .ui
                    .push(crate::states::ui::Notification::info(format!(
                        "{} joined the server.",
                        name
                    )));
            }
            ServerEvent::Quit(name) => {
                self.state.game.online_players_count =
                    self.state.game.online_players_count.saturating_sub(1);
                self.state.game.room_players.retain(|p| p != &name);
                self.state.game.group_members.retain(|p| p != &name);
                self.state
                    .ui
                    .push(crate::states::ui::Notification::info(format!(
                        "{} disconnected.",
                        name
                    )));
            }
            ServerEvent::Room(room_event) => match room_event {
                RoomEvent::PresenceEnter(name) => {
                    if !self.state.game.room_players.contains(&name) {
                        self.state.game.room_players.push(name.clone());
                    }
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} entered the room.",
                            name
                        )));
                }
                RoomEvent::PresenceLeave(name) => {
                    self.state.game.room_players.retain(|p| p != &name);
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} left the room.",
                            name
                        )));
                }
                RoomEvent::Chat(chat) => {
                    self.state.game.chat_history.push(ChatMessage {
                        channel: ChatChannel::Room,
                        sender: chat.sender,
                        content: chat.message,
                    });
                }
            },
            ServerEvent::Group(group_event) => match group_event {
                GroupEvent::Invite(leader) => {
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "You are invited to a group by {}.",
                            leader
                        )));
                }
                GroupEvent::Join(user) => {
                    if !self.state.game.group_members.contains(&user) {
                        self.state.game.group_members.push(user.clone());
                    }
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} joined the group.",
                            user
                        )));
                }
                GroupEvent::Leave(user) => {
                    self.state.game.group_members.retain(|p| p != &user);
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} left the group.",
                            user
                        )));
                }
                GroupEvent::Chat(chat) => {
                    self.state.game.chat_history.push(ChatMessage {
                        channel: ChatChannel::Group,
                        sender: chat.sender,
                        content: chat.message,
                    });
                }
            },
            ServerEvent::GlobalChat(chat) => {
                self.state.game.chat_history.push(ChatMessage {
                    channel: ChatChannel::Global,
                    sender: chat.sender,
                    content: chat.message,
                });
            }
            ServerEvent::PrivateChat(chat) => {
                self.state.game.chat_history.push(ChatMessage {
                    channel: ChatChannel::Private(chat.sender.clone()),
                    sender: chat.sender.clone(),
                    content: chat.message,
                });
                self.state
                    .ui
                    .push(crate::states::ui::Notification::info(format!(
                        "New private message from {}.",
                        chat.sender
                    )));
            }
            ServerEvent::Stats(count) => {
                self.state.game.online_players_count = count;
            }
            ServerEvent::Unknown(raw) => {
                self.state
                    .ui
                    .push(crate::states::ui::Notification::warning(format!(
                        "Unknown event: {}",
                        raw
                    )));
            }
        }
    }
}
