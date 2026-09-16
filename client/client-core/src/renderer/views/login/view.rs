use std::time::Duration;
use crate::collections::Step;
use crate::events::{ApplicationEvent, ConnectionEvent};
use crate::notification::{Notification, NotificationDuration, NotificationTopic};
use crate::renderer::components::{Button, Component, EventFlow, Lifecycle, TextInput};
use crate::renderer::views::login::focus::LoginFocus;
use crate::states::AppState;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tokio::sync::mpsc;

const MAX_PLAYER_NAME_LENGTH: usize = 20;

pub struct LoginView {
    pub focus: LoginFocus,
    pub name_input: TextInput,
    pub ip_input: TextInput,
    pub port_input: TextInput,
    pub connect_button: Button,
    pub quit_button: Button,
}

impl LoginView {
    pub fn new(ip: String, port: String) -> Self {
        let mut view = Self {
            focus: LoginFocus::default(),
            name_input: TextInput::new("Player Name"),
            ip_input: TextInput::new("Server IP"),
            port_input: TextInput::new("Server Port"),
            connect_button: Button::new("Connect"),
            quit_button: Button::new("Quit"),
        };

        view.name_input.set_max_length(MAX_PLAYER_NAME_LENGTH);
        view.ip_input.set_value(ip);
        view.port_input.set_value(port);
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
        self.name_input.blur();
        self.ip_input.blur();
        self.port_input.blur();
        self.connect_button.blur();
        self.quit_button.blur();

        match self.focus {
            LoginFocus::PlayerName => self.name_input.focus(),
            LoginFocus::ServerIp => self.ip_input.focus(),
            LoginFocus::ServerPort => self.port_input.focus(),
            LoginFocus::ConnectButton => self.connect_button.focus(),
            LoginFocus::QuitButton => self.quit_button.focus(),
        }
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
                Constraint::Length(3), // Buttons
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

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(18), // Button width
                Constraint::Percentage(4),  // Spacer
                Constraint::Percentage(18), // Button width
                Constraint::Percentage(30),
            ])
            .split(vertical_chunks[7]);

        self.connect_button.draw(state, frame, button_chunks[3]);
        self.quit_button.draw(state, frame, button_chunks[1]);
    }
}

impl Lifecycle for LoginView {
    fn on_tick(&mut self, state: &mut AppState, sender: &Sender<ApplicationEvent>) {
        self.name_input.on_tick(state, sender);
        self.ip_input.on_tick(state, sender);
        self.port_input.on_tick(state, sender);
    }

    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => {
                if *code == KeyCode::Esc {
                    state.should_quit = true;
                    return EventFlow::Consumed;
                }
                if *code == KeyCode::Tab || *code == KeyCode::Down {
                    self.cycle_focus(Step::Next);
                    return EventFlow::Consumed;
                }
                if *code == KeyCode::BackTab || *code == KeyCode::Up {
                    self.cycle_focus(Step::Previous);
                    return EventFlow::Consumed;
                }

                if *code == KeyCode::Enter
                    && matches!(
                        self.focus,
                        LoginFocus::PlayerName | LoginFocus::ServerIp | LoginFocus::ServerPort
                    )
                {
                    self.set_focus(LoginFocus::ConnectButton);
                    return EventFlow::Consumed;
                }
            }
            CrosstermEvent::Mouse(MouseEvent {
                kind, column, row, ..
            })
                // Mouse navigation (Left click)
                if *kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                    if self.name_input.hit(*column, *row) {
                        self.set_focus(LoginFocus::PlayerName);
                        return EventFlow::Consumed;
                    } else if self.ip_input.hit(*column, *row) {
                        self.set_focus(LoginFocus::ServerIp);
                        return EventFlow::Consumed;
                    } else if self.port_input.hit(*column, *row) {
                        self.set_focus(LoginFocus::ServerPort);
                        return EventFlow::Consumed;
                    } else if self.connect_button.hit(*column, *row) {
                        self.set_focus(LoginFocus::ConnectButton);
                        self.connect_button.press();
                    } else if self.quit_button.hit(*column, *row) {
                        self.set_focus(LoginFocus::QuitButton);
                        self.quit_button.press();
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

                if self.connect_button.take_pressed() {
                    let name = self.name_input.value().to_string();
                    let ip = self.ip_input.value().to_string();
                    let port = self.port_input.value().to_string();

                    if name.is_empty() || ip.is_empty() || port.is_empty() {
                        state.ui.notifications.push(
                            Notification::warning("All fields must be filled")
                                .with_topic(NotificationTopic::Connection),
                        );
                    } else {
                        state.ui.notifications.push(
                            Notification::info("Connecting...")
                                .with_topic(NotificationTopic::Connection)
                                .with_duration(NotificationDuration::Finite(Duration::from_secs(2))),
                        );
                        let _ = event_sender.try_send(ApplicationEvent::Connection(
                            ConnectionEvent::AttemptStarted {
                                server_ip: ip,
                                server_port: port,
                                player_name: name,
                            },
                        ));
                    }
                }

                flow
            }
            LoginFocus::QuitButton => {
                let flow = self
                    .quit_button
                    .handle_device_event(state, event, event_sender);

                if self.quit_button.take_pressed() {
                    state.should_quit = true;
                }

                flow
            }
        }
    }
}
