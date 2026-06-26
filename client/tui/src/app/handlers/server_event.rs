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
                if self.state.game.group_leader.as_ref() == Some(&name) {
                    self.state.game.group_id = None;
                    self.state.game.group_leader = None;
                }
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
                RoomEvent::Take(player, item) => {
                    self.state.game.current_room_items.retain(|id| id != &item);
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} took {}.",
                            player, item
                        )));
                }
                RoomEvent::Drop(player, item) => {
                    self.state.game.current_room_items.push(item.clone());
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} dropped {}.",
                            player, item
                        )));
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
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::info(format!(
                            "{} joined the group.",
                            user
                        )));
                }
                GroupEvent::Leave(user) => {
                    if self.state.game.group_leader.as_ref() == Some(&user) {
                        self.state.game.group_id = None;
                        self.state.game.group_leader = None;
                    }
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
                GroupEvent::Move(direction) => {
                    if let Some(network_manager) = &self.network_manager {
                        let req_look = api_client::protocol::command::enums::ApiRequest::Look(
                            api_client::protocol::command::core::look::LookCommand,
                        );

                        self.state
                            .ui
                            .push(crate::states::ui::Notification::info(format!(
                                "group moved to {}.",
                                direction
                            )));

                        let _ = network_manager.send_command(
                            crate::network::envelopes::RequestEnvelope::new(req_look),
                        );
                    }
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
