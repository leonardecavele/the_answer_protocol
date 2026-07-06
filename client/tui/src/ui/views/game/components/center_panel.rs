use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::scrollable::Scrollable;
use crate::ui::views::game::components::action_history::ActionHistoryComponent;
use crate::ui::views::game::components::inventory::InventoryComponent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc::Sender;

pub struct CenterPanelComponent {
    pub action_history: Scrollable<ActionHistoryComponent>,
    pub inventory: InventoryComponent,
    pub history_area: Option<Rect>,
    pub inventory_area: Option<Rect>,
}

impl CenterPanelComponent {
    pub fn new() -> Self {
        Self {
            action_history: Scrollable::new(ActionHistoryComponent::new()),
            inventory: InventoryComponent::new(),
            history_area: None,
            inventory_area: None,
        }
    }
}

impl Component for CenterPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        self.history_area = Some(chunks[0]);
        self.inventory_area = Some(chunks[1]);

        self.action_history.draw(state, frame, chunks[0]);
        self.inventory.draw(state, frame, chunks[1]);
    }
}

impl Lifecycle for CenterPanelComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.game.ui.current_focus == GameFocus::ActionHistory {
            if self
                .action_history
                .handle_terminal_event(state, event, event_sender)
            {
                return true;
            }
        }
        if state.game.ui.current_focus == GameFocus::InventoryGrid {
            if self
                .inventory
                .handle_terminal_event(state, event, event_sender)
            {
                return true;
            }
        }
        false
    }
}
