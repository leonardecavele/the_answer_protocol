use crate::app::App;
use crate::events::SendEvent;
use crate::notification::{Notification, NotificationTopic};
use client_api::ApiRequest;
use client_api::commands::{DropCommand, MoveCommand, TakeCommand, UseCommand};

impl App {
    pub fn handle_send_event(&mut self, event: SendEvent) {
        match event {
            SendEvent::ApiRequest(request) => self.send(request),
            SendEvent::RawCommand(command) => self.handle_raw_command(command),
        }
    }

    fn handle_raw_command(&mut self, command: String) {
        let hint = match command.trim().to_lowercase().as_str() {
            "help" => Some("Press Ctrl + H to open help."),
            "exit" => Some("Type 'quit' to disconnect from the server."),
            _ => None,
        };

        if let Some(hint) = hint {
            self.record_trace("user input", command);

            self.state
                .ui
                .notifications
                .push(Notification::info(hint.to_string()));

            return;
        }

        if let Some(mut request) = ApiRequest::parse(&command) {
            match &mut request {
                ApiRequest::Drop(DropCommand { item_identifier })
                | ApiRequest::Use(UseCommand { item_identifier }) => {
                    if let Some(item) = self.state.game.player.find_item_by_name(item_identifier) {
                        *item_identifier = item.id.clone();
                    }
                }

                ApiRequest::Take(TakeCommand { item_identifier }) => {
                    if let Some(room) = self.state.game.room.as_mut()
                        && let Some(item) = room.find_item_by_name(item_identifier)
                    {
                        *item_identifier = item.id.clone();
                    }
                }

                ApiRequest::Move(MoveCommand { direction }) => {
                    if let Some(room) = self.state.game.room.as_mut()
                        && let Some(exit_direction) = room.exits.find_exit_by_name(direction)
                    {
                        *direction = exit_direction.key().to_string();
                    }
                }

                _ => {}
            }

            self.record_trace("user input", command);
            self.send(request);
        } else {
            let message = format!("Unknown or invalid command: {}", command);

            self.record_trace("user input", message.clone());

            self.state
                .ui
                .notifications
                .push(Notification::warning(message).with_topic(NotificationTopic::Protocol));
        }
    }
}
