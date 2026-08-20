use super::{ActionHistoryPanel, InventoryPanel};
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::lifecycle::EventFlow;
use crate::ui::components::scrollable::Scrollable;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc::Sender;

pub struct CenterPanel {
    pub action_history: Scrollable<ActionHistoryPanel>,
    pub inventory: InventoryPanel,
    pub history_area: Option<Rect>,
    pub inventory_area: Option<Rect>,
}

impl Default for CenterPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl CenterPanel {
    pub fn new() -> Self {
        Self {
            action_history: Scrollable::new(ActionHistoryPanel::new()),
            inventory: InventoryPanel::new(),
            history_area: None,
            inventory_area: None,
        }
    }
}

impl Component for CenterPanel {
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

impl Lifecycle for CenterPanel {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.focus == GameFocus::ActionHistory
            && self
                .action_history
                .handle_terminal_event(state, event, event_sender)
                .is_consumed()
        {
            return EventFlow::Consumed;
        }
        if state.game.focus == GameFocus::InventoryGrid
            && self
                .inventory
                .handle_terminal_event(state, event, event_sender)
                .is_consumed()
        {
            return EventFlow::Consumed;
        }
        EventFlow::Ignored
    }
}
