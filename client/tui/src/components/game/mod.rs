pub mod context_menu;
pub mod events;
pub mod header;
pub mod input;
pub mod logs;
pub mod scene;

use crate::components::Component;
use crate::events::AppEvent;
use crate::state::{AppState, GameFocus};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub struct GameComponent {
    header: header::HeaderComponent,
    events: events::EventsComponent,
    scene: scene::SceneComponent,
    input: input::InputComponent,
    logs: logs::LogsComponent,
    context_menu: context_menu::ContextMenuComponent,
}

impl GameComponent {
    pub fn new() -> Self {
        Self {
            header: header::HeaderComponent,
            events: events::EventsComponent,
            scene: scene::SceneComponent,
            input: input::InputComponent,
            logs: logs::LogsComponent,
            context_menu: context_menu::ContextMenuComponent,
        }
    }
}

#[async_trait::async_trait]
impl Component for GameComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        tx: &tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if state.ui.context_menu_open {
            self.context_menu.handle_event(state, event, tx).await;
            return;
        }

        if let Event::Key(key) = event {
            if key.code == KeyCode::Tab {
                state.ui.game_focus = match state.ui.game_focus {
                    GameFocus::Input => {
                        if !state.game.npcs_in_room.is_empty() {
                            state.ui.selected_entity_idx = Some(0);
                        }
                        GameFocus::Scene
                    }
                    GameFocus::Scene => GameFocus::SystemLogs,
                    GameFocus::SystemLogs => GameFocus::Input,
                };
                return;
            }
        }

        match state.ui.game_focus {
            GameFocus::Input => self.input.handle_event(state, event, tx).await,
            GameFocus::Scene => self.scene.handle_event(state, event, tx).await,
            GameFocus::SystemLogs => self.logs.handle_event(state, event, tx).await,
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
        self.header.draw(state, f, main_chunks[0]);

        // Middle (Game Events + Scene side-by-side)
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Game Events (Left)
                Constraint::Percentage(50), // Scene (Right)
            ])
            .split(main_chunks[1]);

        self.events.draw(state, f, middle_chunks[0]);
        self.scene.draw(state, f, middle_chunks[1]);

        // Context menu draws over scene
        if state.ui.context_menu_open {
            // Need to pass the inner area of the scene to align correctly, or just the middle_chunks[1]
            // We'll pass the scene's outer chunk, context_menu can offset it.
            self.context_menu.draw(state, f, middle_chunks[1]);
        }

        // Input Area
        self.input.draw(state, f, main_chunks[2]);

        // System Logs Overlay
        self.logs.draw(state, f, area);
    }
}
