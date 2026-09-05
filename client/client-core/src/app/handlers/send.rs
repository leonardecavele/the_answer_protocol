use crate::app::runtime::App;
use crate::events::SendEvent;
use crate::notification::{Notification, NotificationTopic};
use client_api::ApiRequest;

impl App {
    pub fn handle_send_event(&mut self, event: SendEvent) {
        match event {
            SendEvent::ApiRequest(request) => self.send(request),
            SendEvent::RawCommand(command) => self.handle_raw_command(command),
        }
    }

    fn handle_raw_command(&mut self, command: String) {
        if let Some(request) = ApiRequest::parse(&command) {
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
