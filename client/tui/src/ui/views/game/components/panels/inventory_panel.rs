use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{GameFocus, Overlay};
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::theme::panel_block;
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use tokio::sync::mpsc::Sender;

pub const INVENTORY_ITEM_WIDTH: u16 = 20;
pub const INVENTORY_ITEM_HEIGHT: u16 = 4;

pub struct InventoryPanel {
    pub inventory_cols: usize,
    pub inventory_area: Option<Rect>,
}

impl InventoryPanel {
    pub fn new() -> Self {
        Self {
            inventory_cols: 1,
            inventory_area: None,
        }
    }
}

impl Component for InventoryPanel {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.inventory_area = Some(area);

        let inv_block = panel_block(" Inventory ", state.game.focus == GameFocus::InventoryGrid);

        let inv_inner = inv_block.inner(area);
        frame.render_widget(inv_block, area);

        if state.game.player.inventory.is_empty() {
            let p = Paragraph::new(" Your inventory is empty. ").alignment(Alignment::Center);
            frame.render_widget(p, inv_inner);
            return;
        }

        self.inventory_cols = (inv_inner.width / INVENTORY_ITEM_WIDTH) as usize;
        let cols = self.inventory_cols.max(1);

        for (idx, item) in state.game.player.inventory.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;

            let cell_x = inv_inner.x + (col as u16 * INVENTORY_ITEM_WIDTH);
            let cell_y = inv_inner.y + (row as u16 * INVENTORY_ITEM_HEIGHT);

            if cell_y >= inv_inner.bottom() {
                continue; // Cannot fit more rows
            }

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: INVENTORY_ITEM_WIDTH.min(inv_inner.right().saturating_sub(cell_x)),
                height: INVENTORY_ITEM_HEIGHT.min(inv_inner.bottom().saturating_sub(cell_y)),
            };

            let text = format!("{}\n{}", item.name, item.id);
            let mut p_style = Style::default();
            if state.game.focus == GameFocus::InventoryGrid
                && state.game.player.inventory.selected_index() == idx
            {
                p_style = p_style.add_modifier(Modifier::REVERSED).fg(Color::Yellow);
            }

            let mut text_area = cell_area;
            if text_area.height >= 4 {
                text_area.y += 1;
                text_area.height -= 1;
            }

            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(p_style);
            frame.render_widget(paragraph, text_area);
        }
    }
}

impl Lifecycle for InventoryPanel {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.game.focus == GameFocus::InventoryGrid {
            if let crossterm::event::Event::Key(key) = event {
                let inv_count = state.game.player.inventory.len();
                if inv_count > 0 {
                    let cols = self.inventory_cols.max(1);
                    let current = state.game.player.inventory.selected_index();

                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            if current >= cols {
                                state.game.player.inventory.select_index(current - cols);
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            state.game.player.inventory.select_index(current + cols);
                            return true;
                        }
                        crossterm::event::KeyCode::Left => {
                            if current > 0 {
                                state.game.player.inventory.select_index(current - 1);
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Right => {
                            state.game.player.inventory.select_index(current + 1);
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(item) = state.game.player.inventory.selected() {
                                state.game.overlays.open(Overlay::ItemActions {
                                    item_id: item.id.clone(),
                                });
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        false
    }
}
