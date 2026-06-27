use crate::events::{ApplicationEvent, NetworkEvent};
use crate::states::app::AppState;
use crate::states::ui::Notification;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::interactive::Interactive;
use crate::ui::components::widgets::button::ButtonComponent;
use crate::ui::components::widgets::text_input::TextInputComponent;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tokio::sync::mpsc;

#[derive(PartialEq)]
pub enum LoginFocus {
    PlayerName,
    ServerIp,
    ServerPort,
    ConnectButton,
}

pub struct LoginView {
    pub current_focus: LoginFocus,
    pub name_input: Interactive<TextInputComponent>,
    pub ip_input: Interactive<TextInputComponent>,
    pub port_input: Interactive<TextInputComponent>,
    pub connect_button: Interactive<ButtonComponent>,
}

impl LoginView {
    pub fn new(ip: String, port: String) -> Self {
        let mut view = Self {
            current_focus: LoginFocus::PlayerName,
            name_input: Interactive::new(TextInputComponent::new("Player Name")),
            ip_input: Interactive::new(TextInputComponent::new("Server IP")),
            port_input: Interactive::new(TextInputComponent::new("Server Port")),
            connect_button: Interactive::new(ButtonComponent::new("Connect")),
        };
        // Set initial value for defaults
        view.ip_input.inner.value = ip;
        view.port_input.inner.value = port;

        view.update_focus();
        view
    }

    fn update_focus(&mut self) {
        self.name_input.inner.is_focused = self.current_focus == LoginFocus::PlayerName;
        self.ip_input.inner.is_focused = self.current_focus == LoginFocus::ServerIp;
        self.port_input.inner.is_focused = self.current_focus == LoginFocus::ServerPort;
        self.connect_button.inner.is_focused = self.current_focus == LoginFocus::ConnectButton;
    }

    fn cycle_focus_forward(&mut self) {
        self.current_focus = match self.current_focus {
            LoginFocus::PlayerName => LoginFocus::ServerIp,
            LoginFocus::ServerIp => LoginFocus::ServerPort,
            LoginFocus::ServerPort => LoginFocus::ConnectButton,
            LoginFocus::ConnectButton => LoginFocus::PlayerName,
        };
        self.update_focus();
    }

    fn cycle_focus_backward(&mut self) {
        self.current_focus = match self.current_focus {
            LoginFocus::PlayerName => LoginFocus::ConnectButton,
            LoginFocus::ServerIp => LoginFocus::PlayerName,
            LoginFocus::ServerPort => LoginFocus::ServerIp,
            LoginFocus::ConnectButton => LoginFocus::ServerPort,
        };
        self.update_focus();
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

        // Button should be slightly narrower maybe, or just center it
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
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => {
                // Keyboard navigation
                if *code == KeyCode::Tab || *code == KeyCode::Down {
                    self.cycle_focus_forward();
                    return true;
                }
                if *code == KeyCode::BackTab || *code == KeyCode::Up {
                    self.cycle_focus_backward();
                    return true;
                }

                if *code == KeyCode::Enter {
                    if self.current_focus != LoginFocus::ConnectButton {
                        self.current_focus = LoginFocus::ConnectButton;
                        self.update_focus();
                        return true;
                    }
                }
            }
            CrosstermEvent::Mouse(MouseEvent {
                kind, column, row, ..
            }) => {
                // Mouse navigation (Left click)
                if *kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    if self.name_input.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::PlayerName;
                        self.update_focus();
                        return true;
                    } else if self.ip_input.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::ServerIp;
                        self.update_focus();
                        return true;
                    } else if self.port_input.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::ServerPort;
                        self.update_focus();
                        return true;
                    } else if self.connect_button.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::ConnectButton;
                        self.update_focus();
                        // Simulate a button press directly if clicked
                        self.connect_button.inner.is_pressed = true;
                    }
                }
            }
            _ => {}
        }

        match self.current_focus {
            LoginFocus::PlayerName => {
                self.name_input
                    .handle_terminal_event(state, event, event_sender);
            }
            LoginFocus::ServerIp => {
                self.ip_input
                    .handle_terminal_event(state, event, event_sender);
            }
            LoginFocus::ServerPort => {
                self.port_input
                    .handle_terminal_event(state, event, event_sender);
            }
            LoginFocus::ConnectButton => {
                self.connect_button
                    .handle_terminal_event(state, event, event_sender);

                if self.connect_button.inner.take_pressed() {
                    let name = self.name_input.inner.value.clone();
                    let ip = self.ip_input.inner.value.clone();
                    let port = self.port_input.inner.value.clone();

                    if name.is_empty() || ip.is_empty() || port.is_empty() {
                        state
                            .ui
                            .push(Notification::warning("All fields must be filled"));
                    } else {
                        state.ui.push(
                            Notification::info("Connecting...")
                                .with_id(crate::network::manager::NOTIF_ID_CONNECTION_ATTEMPT)
                                .with_duration(60000),
                        );
                        let _ = event_sender.try_send(ApplicationEvent::Network(
                            NetworkEvent::ConnectionAttemptStarted {
                                server_ip: ip,
                                server_port: port,
                                player_name: name,
                            },
                        ));
                    }
                }
            }
        }
        true
    }
}
