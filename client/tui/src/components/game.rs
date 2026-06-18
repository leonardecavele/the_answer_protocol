use crate::commands::handle_command;
use crate::components::Component;
use crate::events::AppEvent;
use crate::state::AppState;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use tokio::sync::mpsc;
use tui_input::Input;

#[derive(PartialEq)]
pub enum GameFocus {
    Input,
    Scene,
    SystemLogs,
}

pub struct GameComponent {
    pub focus: GameFocus,
    pub input: Input,
    pub selected_entity_idx: Option<usize>,
    pub context_menu_open: bool,
    pub context_menu_idx: usize,
}

impl GameComponent {
    pub fn new() -> Self {
        Self {
            focus: GameFocus::Input,
            input: Input::default(),
            selected_entity_idx: None,
            context_menu_open: false,
            context_menu_idx: 0,
        }
    }

    fn get_available_actions(&self, _entity_name: &str) -> Vec<&'static str> {
        vec!["Talk", "Attack"]
    }
}

#[async_trait::async_trait]
impl Component for GameComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        if let Event::Key(key) = event {
            if self.context_menu_open {
                match key.code {
                    KeyCode::Up => {
                        self.context_menu_idx = self.context_menu_idx.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        let entity_name = state
                            .game
                            .npcs_in_room
                            .get(self.selected_entity_idx.unwrap_or(0))
                            .cloned()
                            .unwrap_or_default();
                        let max_idx = self
                            .get_available_actions(&entity_name)
                            .len()
                            .saturating_sub(1);
                        if self.context_menu_idx < max_idx {
                            self.context_menu_idx += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Execute context action
                        let entity_name = state
                            .game
                            .npcs_in_room
                            .get(self.selected_entity_idx.unwrap_or(0))
                            .cloned()
                            .unwrap_or_default();
                        let actions = self.get_available_actions(&entity_name);
                        if let Some(action) = actions.get(self.context_menu_idx) {
                            let cmd =
                                format!("{} {}", action.to_lowercase(), entity_name.to_lowercase());
                            handle_command(state, cmd, tx.clone());
                        }
                        self.context_menu_open = false;
                    }
                    KeyCode::Esc => {
                        self.context_menu_open = false;
                    }
                    _ => {}
                }
                return;
            }

            match key.code {
                KeyCode::Tab => {
                    self.focus = match self.focus {
                        GameFocus::Input => {
                            if !state.game.npcs_in_room.is_empty() {
                                self.selected_entity_idx = Some(0);
                            }
                            GameFocus::Scene
                        }
                        GameFocus::Scene => GameFocus::SystemLogs,
                        GameFocus::SystemLogs => GameFocus::Input,
                    };
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if matches!(self.focus, GameFocus::Scene) {
                        handle_command(state, "move north".to_string(), tx.clone());
                    } else if matches!(self.focus, GameFocus::Input) {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    if matches!(self.focus, GameFocus::Scene) {
                        handle_command(state, "move south".to_string(), tx.clone());
                    } else if matches!(self.focus, GameFocus::Input) {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if matches!(self.focus, GameFocus::Scene) {
                        handle_command(state, "move east".to_string(), tx.clone());
                    } else if matches!(self.focus, GameFocus::Input) {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                }
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    if matches!(self.focus, GameFocus::Scene) {
                        handle_command(state, "move west".to_string(), tx.clone());
                    } else if matches!(self.focus, GameFocus::Input) {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                }
                KeyCode::Up => match self.focus {
                    GameFocus::Scene => {
                        handle_command(state, "move north".to_string(), tx.clone());
                    }
                    GameFocus::Input => {
                        state.ui.game_scroll_offset = state.ui.game_scroll_offset.saturating_add(1);
                    }
                    GameFocus::SystemLogs => {
                        state
                            .ui
                            .logger_state
                            .transition(tui_logger::TuiWidgetEvent::UpKey);
                    }
                },
                KeyCode::Down => match self.focus {
                    GameFocus::Scene => {
                        handle_command(state, "move south".to_string(), tx.clone());
                    }
                    GameFocus::Input => {
                        state.ui.game_scroll_offset = state.ui.game_scroll_offset.saturating_sub(1);
                    }
                    GameFocus::SystemLogs => {
                        state
                            .ui
                            .logger_state
                            .transition(tui_logger::TuiWidgetEvent::DownKey);
                    }
                },
                KeyCode::Left => match self.focus {
                    GameFocus::Scene => {
                        handle_command(state, "move west".to_string(), tx.clone());
                    }
                    GameFocus::SystemLogs => state
                        .ui
                        .logger_state
                        .transition(tui_logger::TuiWidgetEvent::LeftKey),
                    GameFocus::Input => {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                },
                KeyCode::Right => match self.focus {
                    GameFocus::Scene => {
                        handle_command(state, "move east".to_string(), tx.clone());
                    }
                    GameFocus::SystemLogs => state
                        .ui
                        .logger_state
                        .transition(tui_logger::TuiWidgetEvent::RightKey),
                    GameFocus::Input => {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                },
                KeyCode::PageUp => {
                    if matches!(self.focus, GameFocus::Scene) && !state.game.npcs_in_room.is_empty()
                    {
                        let current = self.selected_entity_idx.unwrap_or(0);
                        self.selected_entity_idx = Some(current.saturating_sub(1));
                    }
                }
                KeyCode::PageDown => {
                    if matches!(self.focus, GameFocus::Scene) && !state.game.npcs_in_room.is_empty()
                    {
                        let max = state.game.npcs_in_room.len() - 1;
                        let current = self.selected_entity_idx.unwrap_or(0);
                        if current < max {
                            self.selected_entity_idx = Some(current + 1);
                        }
                    }
                }
                KeyCode::Enter => {
                    if matches!(self.focus, GameFocus::Scene) {
                        if self.selected_entity_idx.is_some() {
                            self.context_menu_open = true;
                            self.context_menu_idx = 0;
                        }
                    } else if matches!(self.focus, GameFocus::Input) {
                        let cmd_str = self.input.value().to_string();
                        self.input.reset();
                        if !cmd_str.trim().is_empty() {
                            handle_command(state, cmd_str, tx.clone());
                        }
                    }
                }
                _ => {
                    if matches!(self.focus, GameFocus::Input) {
                        tui_input::backend::crossterm::EventHandler::handle_event(
                            &mut self.input,
                            event,
                        );
                    }
                }
            }
        } else if let Event::Mouse(_mouse) = event {
            // Basic mouse support
        }
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(0),    // Middle
                Constraint::Length(3), // Input
            ])
            .split(area);

        // Header
        let elapsed = if let Some(t) = state.net.connected_at {
            t.elapsed().as_secs()
        } else {
            0
        };
        let m = elapsed / 60;
        let s = elapsed % 60;

        let player_name = match &state.net.connection_state {
            crate::state::ConnectionState::Connected(n) => n.clone(),
            _ => "Unknown".to_string(),
        };

        let header_str = format!(
            " Time: {:02}:{:02} | Player: {} | Group: {} | HP: {}/{} | Online: {} ",
            m,
            s,
            player_name,
            state.game.group_name.as_deref().unwrap_or("None"),
            state.game.hp,
            state.game.max_hp,
            state.game.online_players
        );
        let header_widget = Paragraph::new(header_str)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(header_widget, main_chunks[0]);

        // Middle (Scene + Game Events taking full width)
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),    // Scene
                Constraint::Length(10), // Game Events
            ])
            .split(main_chunks[1]);

        let unfocused_color = Color::DarkGray;
        let focused_color = Color::Yellow;

        // Scene
        let (_scene_border, scene_style) = if matches!(self.focus, GameFocus::Scene) {
            (
                focused_color,
                Style::default()
                    .fg(focused_color)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (unfocused_color, Style::default().fg(unfocused_color))
        };

        let display_name = if state.game.current_room_name.is_empty() {
            "Unknown Location".to_string()
        } else {
            state.game.current_room_name.clone()
        };
        let scene_title = format!(" Scene: {} ", display_name);
        let scene_block = Block::default()
            .borders(Borders::ALL)
            .border_style(scene_style)
            .title(scene_title);
        let inner_scene = scene_block.inner(left_chunks[0]);
        f.render_widget(scene_block, left_chunks[0]);

        // Fallback to "default" room if current_room is empty
        let check_room = if state.game.current_room.is_empty() {
            "default"
        } else {
            &state.game.current_room
        };

        if let Some(dyn_img) = state
            .assets
            .get_image_for_context(check_room, &state.game.npcs_in_room)
        {
            let img_widget = ratatui_image::StatefulImage::new();
            f.render_stateful_widget(img_widget, inner_scene, dyn_img);
        } else {
            f.render_widget(
                Paragraph::new(">> IMAGE NOT FOUND / FAILED TO LOAD <<")
                    .style(Style::default().fg(Color::Red)),
                inner_scene,
            );
        }

        // Scene Overlays (NPC List, Arrows, etc)

        // Draw directional arrows if in Scene focus
        if matches!(self.focus, GameFocus::Scene) {
            let north = Paragraph::new("[N]orth")
                .style(Style::default().fg(Color::Yellow).bg(Color::Black));
            let south = Paragraph::new("[S]outh")
                .style(Style::default().fg(Color::Yellow).bg(Color::Black));
            let east =
                Paragraph::new("[E]ast").style(Style::default().fg(Color::Yellow).bg(Color::Black));
            let west =
                Paragraph::new("[W]est").style(Style::default().fg(Color::Yellow).bg(Color::Black));

            // Positioning arrows manually over the inner scene
            let mut n_area = inner_scene;
            n_area.height = 1;
            n_area.width = 7;
            n_area.x = inner_scene.x + (inner_scene.width / 2).saturating_sub(3);
            n_area.y = inner_scene.y;

            let mut s_area = inner_scene;
            s_area.height = 1;
            s_area.width = 7;
            s_area.x = inner_scene.x + (inner_scene.width / 2).saturating_sub(3);
            s_area.y = inner_scene.y + inner_scene.height.saturating_sub(1);

            let mut e_area = inner_scene;
            e_area.height = 1;
            e_area.width = 6;
            e_area.x = inner_scene.x + inner_scene.width.saturating_sub(6);
            e_area.y = inner_scene.y + (inner_scene.height / 2);

            let mut w_area = inner_scene;
            w_area.height = 1;
            w_area.width = 6;
            w_area.x = inner_scene.x;
            w_area.y = inner_scene.y + (inner_scene.height / 2);

            f.render_widget(north, n_area);
            f.render_widget(south, s_area);
            f.render_widget(east, e_area);
            f.render_widget(west, w_area);
        }

        if !state.game.npcs_in_room.is_empty() {
            let entities_str = state
                .game
                .npcs_in_room
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let dname = state.assets.get_display_name(n);
                    if Some(i) == self.selected_entity_idx {
                        format!("[{}]", dname)
                    } else {
                        dname
                    }
                })
                .collect::<Vec<_>>()
                .join("  ");

            let entities_p = Paragraph::new(format!(" Entities: {} ", entities_str)).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(
                entities_p,
                Rect {
                    x: inner_scene.x,
                    y: inner_scene.y + inner_scene.height.saturating_sub(1),
                    width: inner_scene.width,
                    height: 1,
                },
            );
        }

        // Context Menu
        if self.context_menu_open {
            if let Some(idx) = self.selected_entity_idx {
                if let Some(entity_name) = state.game.npcs_in_room.get(idx) {
                    let actions = self.get_available_actions(entity_name);
                    let items: Vec<ListItem> = actions
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            if i == self.context_menu_idx {
                                ListItem::new(format!("> {}", a))
                                    .style(Style::default().fg(Color::Yellow))
                            } else {
                                ListItem::new(format!("  {}", a))
                            }
                        })
                        .collect();
                    let menu = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" Action "));

                    let menu_rect = Rect {
                        x: inner_scene.x + 2,
                        y: inner_scene.y + 2,
                        width: 15,
                        height: actions.len() as u16 + 2,
                    };
                    f.render_widget(Clear, menu_rect);
                    f.render_widget(menu, menu_rect);
                }
            }
        }

        // Game Events
        let game_lines: Vec<Line> = state
            .game
            .game_output
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect();
        let max_scroll =
            (game_lines.len() as u16).saturating_sub(left_chunks[1].height.saturating_sub(2));
        let scroll = max_scroll.saturating_sub(state.ui.game_scroll_offset);

        // Game Events is never "focused" in the tab cycle directly (Input scrolls it), but we can give it unfocused_color
        let messages_widget = Paragraph::new(game_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(unfocused_color))
                    .title(" Game Events "),
            )
            .scroll((scroll, 0))
            .wrap(Wrap { trim: true });
        f.render_widget(messages_widget, left_chunks[1]);

        // Input Area
        let input_style = if matches!(self.focus, GameFocus::Input) {
            Style::default()
                .fg(focused_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(unfocused_color)
        };
        let input_widget = Paragraph::new(self.input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(input_style)
                    .title(" Command Input [Tab to switch] "),
            )
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(input_widget, main_chunks[2]);

        // System Logs Overlay
        if matches!(self.focus, GameFocus::SystemLogs) {
            let overlay_width = area.width.saturating_mul(80) / 100;
            let overlay_height = area.height.saturating_mul(80) / 100;
            let overlay_x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
            let overlay_y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
            let overlay_rect = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

            let logs_widget = tui_logger::TuiLoggerWidget::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(
                            Style::default()
                                .fg(focused_color)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" System Logs [Tab to hide] "),
                )
                .state(&state.ui.logger_state)
                .style_error(Style::default().fg(Color::Red))
                .style_info(Style::default().fg(Color::Blue));

            f.render_widget(Clear, overlay_rect);
            f.render_widget(logs_widget, overlay_rect);
        }
    }
}
