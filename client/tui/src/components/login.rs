use crate::components::Component;
use crate::events::AppEvent;
use crate::state::AppState;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::sync::mpsc;
use tui_input::Input;

#[derive(PartialEq, Clone)]
pub enum LoginField {
    Username,
    Address,
    Port,
    Button,
}

pub struct LoginComponent {
    pub login_field: LoginField,
    pub username_input: Input,
    pub address_input: Input,
    pub port_input: Input,
}

impl LoginComponent {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            login_field: LoginField::Username,
            username_input: Input::default(),
            address_input: Input::from(ip),
            port_input: Input::from(port),
        }
    }

    fn submit(&self, state: &mut AppState, tx: &mpsc::Sender<AppEvent>) {
        if self.username_input.value().trim().is_empty() {
            state.ui.push_notification(
                "Username cannot be empty!".to_string(),
                crate::state::NotificationType::Error,
                16,
            );
            return;
        }
        state.net.server_ip = self.address_input.value().to_string();
        state.net.server_port = self.port_input.value().to_string();

        state.net.connection_state = crate::state::ConnectionState::Connecting;
        let _ = tx.send(AppEvent::Network(crate::events::NetEvent::AttemptConnect(
            self.username_input.value().to_string(),
            self.address_input.value().to_string(),
            self.port_input.value().to_string(),
        )));
    }
}

#[async_trait::async_trait]
impl Component for LoginComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        tx: &mpsc::Sender<AppEvent>,
    ) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Tab => {
                    self.login_field = match self.login_field {
                        LoginField::Username => LoginField::Address,
                        LoginField::Address => LoginField::Port,
                        LoginField::Port => LoginField::Button,
                        LoginField::Button => LoginField::Username,
                    };
                }
                KeyCode::BackTab => {
                    self.login_field = match self.login_field {
                        LoginField::Username => LoginField::Button,
                        LoginField::Address => LoginField::Username,
                        LoginField::Port => LoginField::Address,
                        LoginField::Button => LoginField::Port,
                    };
                }
                KeyCode::Up => {
                    self.login_field = match self.login_field {
                        LoginField::Username => LoginField::Button,
                        LoginField::Address => LoginField::Username,
                        LoginField::Port => LoginField::Address,
                        LoginField::Button => LoginField::Port,
                    };
                }
                KeyCode::Down => {
                    self.login_field = match self.login_field {
                        LoginField::Username => LoginField::Address,
                        LoginField::Address => LoginField::Port,
                        LoginField::Port => LoginField::Button,
                        LoginField::Button => LoginField::Username,
                    };
                }
                KeyCode::Enter => {
                    if matches!(self.login_field, LoginField::Button) {
                        self.submit(state, tx);
                    } else {
                        self.login_field = LoginField::Button;
                    }
                }
                _ => match self.login_field {
                    LoginField::Username => {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.username_input,
                            event,
                        );
                    }
                    LoginField::Address => {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.address_input,
                            event,
                        );
                    }
                    LoginField::Port => {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.port_input,
                            event,
                        );
                    }
                    LoginField::Button => {}
                },
            }
        }
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        // We can reuse the UI code from `ui/login.rs`
        // But since we are decoupling, we bring the drawing logic here.
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Length(16),
                Constraint::Percentage(25),
            ])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(vertical_chunks[1]);

        let center_rect = horizontal_chunks[1];

        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(" The Answer Protocol ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));

        if matches!(
            state.net.connection_state,
            crate::state::ConnectionState::Connecting
        ) {
            block = block.style(Style::default().fg(Color::Yellow));
        }

        f.render_widget(block, center_rect);

        let inner_rect = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Username
                Constraint::Length(3), // IP
                Constraint::Length(3), // Port
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Button
            ])
            .split(center_rect);

        // Username
        let user_style = if matches!(self.login_field, LoginField::Username) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let user_p = Paragraph::new(self.username_input.value())
            .block(Block::default().borders(Borders::ALL).title(" Username "))
            .style(user_style);
        f.render_widget(user_p, inner_rect[0]);

        // Address
        let addr_style = if matches!(self.login_field, LoginField::Address) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let addr_p = Paragraph::new(self.address_input.value())
            .block(Block::default().borders(Borders::ALL).title(" Server IP "))
            .style(addr_style);
        f.render_widget(addr_p, inner_rect[1]);

        // Port
        let port_style = if matches!(self.login_field, LoginField::Port) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let port_p = Paragraph::new(self.port_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Server Port "),
            )
            .style(port_style);
        f.render_widget(port_p, inner_rect[2]);

        // Button
        let btn_style = if matches!(self.login_field, LoginField::Button) {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };

        let status_text = match state.net.connection_state {
            crate::state::ConnectionState::Connecting => " CONNECTING... ",
            _ => " CONNECT ",
        };

        let btn_p = Paragraph::new(status_text)
            .alignment(Alignment::Center)
            .style(btn_style);
        f.render_widget(btn_p, inner_rect[4]);
    }
}
