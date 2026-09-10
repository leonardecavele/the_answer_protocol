use crate::collections::{SelectableList, Step};
use crate::events::{ApplicationEvent, SendEvent};
use crate::renderer::components::{
    Component, EventFlow, LabelButton, Lifecycle, is_mouse_in_rect, scroll_direction,
};
use crate::renderer::theme::{SURFACE_COLOR, panel_block, selection_style};
use crate::states::AppState;
use crate::states::game::{GameFocus, ItemActionsState, ItemLocation, ItemStack, Overlay};
use client_api::ApiRequest;
use client_api::commands::InventoryCommand;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
};
use tokio::sync::mpsc::Sender;

const INVENTORY_ITEM_WIDTH: u16 = 20;
const INVENTORY_ITEM_HEIGHT: u16 = 4;

pub enum InventoryPanelHit {
    Item(Option<usize>),
    None,
}

pub struct InventoryPanel {
    cols: usize,
    rows: usize,
    area: Option<Rect>,
    refresh_button: LabelButton,
}

impl Default for InventoryPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryPanel {
    pub fn new() -> Self {
        Self {
            cols: 1,
            rows: 1,
            area: None,
            refresh_button: LabelButton::new("INVENTORY"),
        }
    }

    pub fn hit(&self, state: &AppState, column: u16, row: u16) -> InventoryPanelHit {
        if let Some(area) = self.area
            && is_mouse_in_rect(column, row, area)
        {
            let rel_x = column.saturating_sub(area.x);
            let rel_y = row.saturating_sub(area.y);
            if rel_x > 0 && rel_y > 0 {
                let col = (rel_x - 1) as usize / INVENTORY_ITEM_WIDTH as usize;
                let row = (rel_y - 1) as usize / INVENTORY_ITEM_HEIGHT as usize;
                let cols = self.cols.max(1);
                let index = state.game.player.inventory.offset() + row * cols + col;
                return InventoryPanelHit::Item(Some(index));
            }

            return InventoryPanelHit::Item(None);
        }

        InventoryPanelHit::None
    }

    fn visible_count(&self) -> usize {
        self.rows * self.cols
    }

    fn adjust_offset_alignment(&self, inventory: &mut SelectableList<ItemStack>) {
        let cols = self.cols.max(1);
        let misalignment = inventory.offset() % cols;

        let Some(selected) = inventory.selected_index() else {
            return;
        };

        if misalignment == 0 {
            return;
        }

        if selected < inventory.offset() - misalignment + self.visible_count() {
            inventory.scroll(Step::Previous, misalignment);
        } else {
            inventory.scroll(Step::Next, cols - misalignment);
        }
    }
}

impl Component for InventoryPanel {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.area = Some(area);

        let focused = state.game.focus() == GameFocus::InventoryGrid;
        let inv_block = panel_block(" Inventory ", focused);

        let inv_inner = inv_block.inner(area);
        frame.render_widget(inv_block, area);

        let button_width = self.refresh_button.width();

        if area.width > button_width + 2 {
            let button_area = Rect::new(area.right() - button_width - 1, area.y, button_width, 1);
            self.refresh_button.draw(frame, button_area);
        } else {
            self.refresh_button.hide();
        }

        if state.game.player.inventory.is_empty() {
            let p = Paragraph::new(" Your inventory is empty. ").alignment(Alignment::Center);
            frame.render_widget(p, inv_inner);
            return;
        }

        self.cols = (inv_inner.width / INVENTORY_ITEM_WIDTH) as usize;
        self.rows = (inv_inner.height / INVENTORY_ITEM_HEIGHT) as usize;

        let cols = self.cols.max(1);
        let offset = state.game.player.inventory.offset();

        for (idx, stack) in state.game.player.inventory.iter().enumerate().skip(offset) {
            let col = (idx - offset) % cols;
            let row = (idx - offset) / cols;

            let cell_x = inv_inner.x + (col as u16 * INVENTORY_ITEM_WIDTH);
            let cell_y = inv_inner.y + (row as u16 * INVENTORY_ITEM_HEIGHT);

            if cell_y >= inv_inner.bottom() {
                continue;
            }

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: INVENTORY_ITEM_WIDTH.min(inv_inner.right().saturating_sub(cell_x)),
                height: INVENTORY_ITEM_HEIGHT.min(inv_inner.bottom().saturating_sub(cell_y)),
            };

            let text = format!("{}\nx{}", stack.name, stack.len());
            let style = selection_style(
                Color::Reset,
                focused && state.game.player.inventory.is_selected(idx),
            );

            let text_area = cell_area.inner(Margin::new(1, 1));

            frame.render_widget(
                Block::default().style(Style::default().bg(SURFACE_COLOR)),
                text_area,
            );

            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(style);
            frame.render_widget(paragraph, text_area);
        }
    }
}

impl Lifecycle for InventoryPanel {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        state
            .game
            .player
            .inventory
            .set_visible_count(self.visible_count());

        if let crossterm::event::Event::Mouse(mouse) = event
            && mouse.kind
                == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
            && self.refresh_button.hit(mouse.column, mouse.row)
        {
            let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(
                ApiRequest::Inventory(InventoryCommand),
            )));
            return EventFlow::Consumed;
        }

        if let crossterm::event::Event::Mouse(mouse) = event
            && let Some(step) = scroll_direction(mouse.kind)
            && let InventoryPanelHit::Item(_) = self.hit(state, mouse.column, mouse.row)
        {
            state.game.player.inventory.scroll(step, self.cols.max(1));
            return EventFlow::Consumed;
        }

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

                        self.adjust_offset_alignment(&mut state.game.player.inventory);

                        return EventFlow::Consumed;
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some(item) = state
                            .game
                            .player
                            .inventory
                            .selected()
                            .and_then(|stack| stack.first())
                        {
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
