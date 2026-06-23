use crate::events::types::NotificationType;
use crate::states::app::AppState;
use crate::states::ui::Notification;
use crate::ui::components::button::ButtonComponent;
use crate::ui::components::text_input::TextInputComponent;
use crate::ui::components::Component;
use crate::ui::AppView;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

#[derive(PartialEq)]
pub enum LoginFocus {
    PlayerName,
    ServerIp,
    ServerPort,
    ConnectButton,
}

pub struct LoginView {
    pub current_focus: LoginFocus,
    pub name_input: TextInputComponent,
    pub ip_input: TextInputComponent,
    pub port_input: TextInputComponent,
    pub connect_button: ButtonComponent,
}

impl LoginView {
    pub fn new() -> Self {
        let mut view = Self {
            current_focus: LoginFocus::PlayerName,
            name_input: TextInputComponent::new("Player Name"),
            ip_input: TextInputComponent::new("Server IP"),
            port_input: TextInputComponent::new("Server Port"),
            connect_button: ButtonComponent::new("Connect"),
        };
        // Set initial value for defaults
        view.ip_input.value = "127.0.0.1".to_string();
        view.port_input.value = "38800".to_string();
        
        view.update_focus();
        view
    }

    fn update_focus(&mut self) {
        self.name_input.is_focused = self.current_focus == LoginFocus::PlayerName;
        self.ip_input.is_focused = self.current_focus == LoginFocus::ServerIp;
        self.port_input.is_focused = self.current_focus == LoginFocus::ServerPort;
        self.connect_button.is_focused = self.current_focus == LoginFocus::ConnectButton;
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

impl AppView for LoginView {
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

    fn handle_event(&mut self, state: &mut AppState, event: &CrosstermEvent) {
        match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => {
                // Keyboard navigation
                if *code == KeyCode::Tab || *code == KeyCode::Down {
                    self.cycle_focus_forward();
                    return;
                }
                if *code == KeyCode::BackTab || *code == KeyCode::Up {
                    self.cycle_focus_backward();
                    return;
                }

                if *code == KeyCode::Enter {
                    if self.current_focus != LoginFocus::ConnectButton {
                        self.current_focus = LoginFocus::ConnectButton;
                        self.update_focus();
                        return;
                    }
                }
            }
            CrosstermEvent::Mouse(MouseEvent { kind, column, row, .. }) => {
                // Mouse navigation (Left click)
                if *kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    if self.name_input.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::PlayerName;
                        self.update_focus();
                        return;
                    } else if self.ip_input.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::ServerIp;
                        self.update_focus();
                        return;
                    } else if self.port_input.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::ServerPort;
                        self.update_focus();
                        return;
                    } else if self.connect_button.is_mouse_over(*column, *row) {
                        self.current_focus = LoginFocus::ConnectButton;
                        self.update_focus();
                        // Simulate a button press directly if clicked
                        self.connect_button.is_pressed = true;
                    }
                }
            }
            _ => {}
        }

        // Delegate event to the currently focused component
        match self.current_focus {
            LoginFocus::PlayerName => {
                self.name_input.handle_event(state, event);
            }
            LoginFocus::ServerIp => {
                self.ip_input.handle_event(state, event);
            }
            LoginFocus::ServerPort => {
                self.port_input.handle_event(state, event);
            }
            LoginFocus::ConnectButton => {
                self.connect_button.handle_event(state, event);
                
                if self.connect_button.take_pressed() {
                    // Check if fields are empty
                    if self.name_input.value.is_empty() 
                        || self.ip_input.value.is_empty() 
                        || self.port_input.value.is_empty() 
                    {
                        state.ui.notifications.push(Notification::new(
                            None, 
                            "All fields are required!".to_string(), 
                            NotificationType::Error, 
                            5000
                        ));
                    } else {
                        // Fields are valid, we update the state and we would normally trigger the connection here
                        state.game.player_name = Some(self.name_input.value.clone());
                        state.network.server_ip = self.ip_input.value.clone();
                        state.network.server_port = self.port_input.value.clone();
                        
                        state.ui.notifications.push(Notification::new(
                            None, 
                            format!("Connecting to {}:{}...", state.network.server_ip, state.network.server_port), 
                            NotificationType::Information, 
                            5000
                        ));
                        
                        // NOTE: Real network trigger will be implemented later.
                        // Right now the NetworkManager in app.rs starts automatically.
                    }
                }
            }
        }
    }
}
