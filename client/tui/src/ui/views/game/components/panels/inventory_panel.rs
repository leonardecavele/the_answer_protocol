use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{GameFocus, ItemActionsState, ItemLocation, Overlay};
use crate::ui::components::{Component, EventFlow, Lifecycle, is_mouse_in_rect};
use crate::ui::theme::{panel_block, selection_style};
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use ratatui::{Frame, layout::Rect, style::Color};
use tokio::sync::mpsc::Sender;

const INVENTORY_ITEM_WIDTH: u16 = 20;
const INVENTORY_ITEM_HEIGHT: u16 = 4;

pub enum InventoryPanelHit {
    Item(Option<usize>),
    None,
}

#[derive(Default)]
pub struct InventoryPanel {
    cols: usize,
    area: Option<Rect>,
}

impl InventoryPanel {
    pub fn new() -> Self {
        Self {
            cols: 1,
            area: None,
        }
    }

    pub fn hit(&self, column: u16, row: u16) -> InventoryPanelHit {
        if let Some(area) = self.area
            && is_mouse_in_rect(column, row, area)
        {
            let rel_x = column.saturating_sub(area.x);
            let rel_y = row.saturating_sub(area.y);
            if rel_x > 0 && rel_y > 0 {
                let col = (rel_x - 1) as usize / INVENTORY_ITEM_WIDTH as usize;
                let row = (rel_y - 1) as usize / INVENTORY_ITEM_HEIGHT as usize;
                let cols = self.cols.max(1);
                let index = row * cols + col;
                return InventoryPanelHit::Item(Some(index));
            }

            return InventoryPanelHit::Item(None);
        }

        InventoryPanelHit::None
    }
}

impl Component for InventoryPanel {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.area = Some(area);

        let focused = state.game.focus() == GameFocus::InventoryGrid;
        let inv_block = panel_block(" Inventory ", focused);

        let inv_inner = inv_block.inner(area);
        frame.render_widget(inv_block, area);

        if state.game.player.inventory.is_empty() {
            let p = Paragraph::new(" Your inventory is empty. ").alignment(Alignment::Center);
            frame.render_widget(p, inv_inner);
            return;
        }

        self.cols = (inv_inner.width / INVENTORY_ITEM_WIDTH) as usize;
        let cols = self.cols.max(1);

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
            let style = selection_style(
                Color::Reset,
                focused && state.game.player.inventory.is_selected(idx),
            );

            let mut text_area = cell_area;
            if text_area.height >= 4 {
                text_area.y += 1;
                text_area.height -= 1;
            }

            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(style);
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
    ) -> EventFlow {
        if state.game.focus() == GameFocus::InventoryGrid
            && let crossterm::event::Event::Key(key) = event
        {
            let inv_count = state.game.player.inventory.len();
            if inv_count > 0 {
                let cols = self.cols.max(1);

                match key.code {
                    crossterm::event::KeyCode::Up
                    | crossterm::event::KeyCode::Down
                    | crossterm::event::KeyCode::Left
                    | crossterm::event::KeyCode::Right => {
                        let inventory = &mut state.game.player.inventory;

                        match inventory.selected_index() {
                            None => inventory.select_index(0),
                            Some(current) => match key.code {
                                crossterm::event::KeyCode::Up if current >= cols => {
                                    inventory.select_index(current - cols)
                                }
                                crossterm::event::KeyCode::Down => {
                                    inventory.select_index(current + cols)
                                }
                                crossterm::event::KeyCode::Left if current > 0 => {
                                    inventory.select_index(current - 1)
                                }
                                crossterm::event::KeyCode::Right => {
                                    inventory.select_index(current + 1)
                                }
                                _ => {}
                            },
                        }

                        return EventFlow::Consumed;
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some(item) = state.game.player.inventory.selected() {
                            let item_id = item.id.clone();
                            state
                                .game
                                .overlays
                                .open(Overlay::ItemActions(ItemActionsState::new(
                                    item_id,
                                    ItemLocation::Inventory,
                                )));
                            return EventFlow::Consumed;
                        }
                    }
                    _ => {}
                }
            }
        }
        EventFlow::Ignored
    }
}
