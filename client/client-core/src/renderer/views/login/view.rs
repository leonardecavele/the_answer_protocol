use crate::collections::Step;
use crate::events::{ApplicationEvent, NetworkConnectionEvent};
use crate::notification::{Notification, NotificationTopic};
use crate::renderer::components::{
    Button, Component, EventFlow, Interactive, Lifecycle, TextInput,
};
use crate::renderer::views::login::focus::LoginFocus;
use crate::states::AppState;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tokio::sync::mpsc;

pub struct LoginView {
    pub focus: LoginFocus,
    pub name_input: Interactive<TextInput>,
    pub ip_input: Interactive<TextInput>,
    pub port_input: Interactive<TextInput>,
    pub connect_button: Interactive<Button>,
}

impl LoginView {
    pub fn new(ip: String, port: String) -> Self {
        let mut view = Self {
            focus: LoginFocus::default(),
            name_input: Interactive::new(TextInput::new("Player Name")),
            ip_input: Interactive::new(TextInput::new("Server IP")),
            port_input: Interactive::new(TextInput::new("Server Port")),
            connect_button: Interactive::new(Button::new("Connect")),
        };

        view.ip_input.inner.value = ip;
        view.port_input.inner.value = port;
        view.update_focus();

        view
    }

    fn set_focus(&mut self, focus: LoginFocus) {
        self.focus = focus;
        self.update_focus();
    }

    fn cycle_focus(&mut self, step: Step) {
        match step {
            Step::Next => self.focus.next(),
            Step::Previous => self.focus.prev(),
        }
        self.update_focus();
    }

    fn update_focus(&mut self) {
        self.name_input.inner.is_focused = self.focus == LoginFocus::PlayerName;
        self.ip_input.inner.is_focused = self.focus == LoginFocus::ServerIp;
        self.port_input.inner.is_focused = self.focus == LoginFocus::ServerPort;
        self.connect_button.inner.is_focused = self.focus == LoginFocus::ConnectButton;
    }
}

impl Component for LoginView {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        // Create a centered layout
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Length(3), // Name input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // IP input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Port input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Button
                Constraint::Percentage(30),
            ])
            .split(area);

        let get_center_rect = |row_index: usize| -> Rect {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(vertical_chunks[row_index])[1]
        };

        self.name_input.draw(state, frame, get_center_rect(1));
        self.ip_input.draw(state, frame, get_center_rect(3));
        self.port_input.draw(state, frame, get_center_rect(5));

        let button_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20), // Button width
                Constraint::Percentage(40),
            ])
            .split(vertical_chunks[7])[1];

        self.connect_button.draw(state, frame, button_area);
    }
}

impl Lifecycle for LoginView {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => {
                if *code == KeyCode::Tab || *code == KeyCode::Down {
                    self.cycle_focus(Step::Next);
                    return EventFlow::Consumed;
                }
                if *code == KeyCode::BackTab || *code == KeyCode::Up {
                    self.cycle_focus(Step::Previous);
                    return EventFlow::Consumed;
                }

                if *code == KeyCode::Enter
                    && self.focus != LoginFocus::ConnectButton {
                    self.set_focus(LoginFocus::ConnectButton);
                        return EventFlow::Consumed;
                    }
            }
            CrosstermEvent::Mouse(MouseEvent {
                kind, column, row, ..
            })
                // Mouse navigation (Left click)
                if *kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                    if self.name_input.is_mouse_over(*column, *row) {
                        self.set_focus(LoginFocus::PlayerName);
                        return EventFlow::Consumed;
                    } else if self.ip_input.is_mouse_over(*column, *row) {
                        self.set_focus(LoginFocus::ServerIp);
                        return EventFlow::Consumed;
                    } else if self.port_input.is_mouse_over(*column, *row) {
                        self.set_focus(LoginFocus::ServerPort);
                        return EventFlow::Consumed;
                    } else if self.connect_button.is_mouse_over(*column, *row) {
                        self.set_focus(LoginFocus::ConnectButton);
                        self.connect_button.inner.is_pressed = true;
                    }
                }
            _ => {}
        }

        match self.focus {
            LoginFocus::PlayerName => {
                self.name_input
                    .handle_device_event(state, event, event_sender)
            }
            LoginFocus::ServerIp => self
                .ip_input
                .handle_device_event(state, event, event_sender),
            LoginFocus::ServerPort => {
                self.port_input
                    .handle_device_event(state, event, event_sender)
            }
            LoginFocus::ConnectButton => {
                let flow = self
                    .connect_button
                    .handle_device_event(state, event, event_sender);

                if self.connect_button.inner.take_pressed() {
                    let name = self.name_input.inner.value.clone();
                    let ip = self.ip_input.inner.value.clone();
                    let port = self.port_input.inner.value.clone();

                    if name.is_empty() || ip.is_empty() || port.is_empty() {
                        state.ui.notifications.push(
                            Notification::warning("All fields must be filled")
                                .with_topic(NotificationTopic::Connection),
                        );
                    } else {
                        state.ui.notifications.push(
                            Notification::info("Connecting...")
                                .with_topic(NotificationTopic::Connection)
                                .with_ms(60_000),
                        );
                        let _ = event_sender.try_send(ApplicationEvent::Network(
                            NetworkConnectionEvent::AttemptStarted {
                                server_ip: ip,
                                server_port: port,
                                player_name: name,
                            },
                        ));
                    }
                }

                flow
            }
        }
    }
}
