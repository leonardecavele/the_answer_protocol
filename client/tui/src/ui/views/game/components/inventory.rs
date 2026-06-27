use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::ui::GameFocus;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::theme::default_block;
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use ratatui_image::{Resize, StatefulImage};
use tokio::sync::mpsc::Sender;

const INVENTORY_ITEM_WIDTH: u16 = 20;
const INVENTORY_ITEM_HEIGHT: u16 = 10;

pub struct InventoryComponent {
    pub inventory_cols: usize,
    pub inventory_area: Option<Rect>,
}

impl InventoryComponent {
    pub fn new() -> Self {
        Self {
            inventory_cols: 1,
            inventory_area: None,
        }
    }
}

impl Component for InventoryComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.inventory_area = Some(area);

        let mut inv_block = default_block().title(" Inventory ");
        if state.ui.current_focus == GameFocus::InventoryGrid {
            inv_block = inv_block.border_style(Style::default().fg(Color::Yellow));
        }

        let inv_inner = inv_block.inner(area);
        frame.render_widget(inv_block, area);

        if state.game.inventory.is_empty() {
            let p = Paragraph::new(" Your inventory is empty. ").alignment(Alignment::Center);
            frame.render_widget(p, inv_inner);
            return;
        }

        let cols = (inv_inner.width / INVENTORY_ITEM_WIDTH).max(1) as usize;
        self.inventory_cols = cols;

        for (idx, item_id) in state.game.inventory.iter().enumerate() {
            let row = idx / cols;
            let col = idx % cols;

            let cell_x = inv_inner.x + (col as u16) * INVENTORY_ITEM_WIDTH;
            let cell_y = inv_inner.y + (row as u16) * INVENTORY_ITEM_HEIGHT;

            if cell_y >= inv_inner.bottom() {
                break;
            }

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: INVENTORY_ITEM_WIDTH.min(inv_inner.right().saturating_sub(cell_x)),
                height: INVENTORY_ITEM_HEIGHT.min(inv_inner.bottom().saturating_sub(cell_y)),
            };

            let text_height = 2;
            let image_area = Rect {
                x: cell_area.x,
                y: cell_area.y,
                width: cell_area.width,
                height: cell_area.height.saturating_sub(text_height),
            };
            let text_area = Rect {
                x: cell_area.x,
                y: cell_area.y + image_area.height,
                width: cell_area.width,
                height: text_height.min(cell_area.height.saturating_sub(image_area.height)),
            };

            let mut path_to_load = None;
            let display_name = if let Some(item) = state.game.manifest.items.get(item_id) {
                if let Some(path) = &item.image_path {
                    path_to_load = Some(path.clone());
                }
                item.name.clone()
            } else {
                item_id.clone()
            };

            if let Some(path) = path_to_load {
                let mut cache = state.ui.image_cache.borrow_mut();
                if !cache.contains_key(&path) {
                    if let Ok(dyn_img) = image::open(&path) {
                        let width = dyn_img.width();
                        let height = dyn_img.height();
                        let protocol = state.ui.image_picker.new_resize_protocol(dyn_img);
                        cache.insert(path.clone(), Some((protocol, width, height)));
                    } else {
                        cache.insert(path.clone(), None);
                    }
                }
                if let Some(Some((protocol, _, _))) = cache.get_mut(&path) {
                    let image_widget = StatefulImage::default().resize(Resize::Fit(None));
                    frame.render_stateful_widget(image_widget, image_area, protocol);
                }
            }

            let text = format!("{}\n{}", display_name, item_id);
            let mut p_style = Style::default();
            if state.ui.current_focus == GameFocus::InventoryGrid
                && state.game.inventory_cursor == idx
            {
                p_style = p_style.add_modifier(Modifier::REVERSED).fg(Color::Yellow);
            }
            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(p_style);
            frame.render_widget(paragraph, text_area);
        }
    }
}

impl Lifecycle for InventoryComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.ui.current_focus == GameFocus::InventoryGrid {
            if let crossterm::event::Event::Key(key) = event {
                let inv_count = state.game.inventory.len();
                if inv_count > 0 {
                    let cols = self.inventory_cols.max(1);
                    let rows = (inv_count + cols - 1) / cols;
                    let current = state.game.inventory_cursor;
                    let current_row = current / cols;

                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            if current_row > 0 {
                                state.game.inventory_cursor = current - cols;
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            if current_row + 1 < rows {
                                let target = current + cols;
                                state.game.inventory_cursor = target.min(inv_count - 1);
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Left => {
                            if current > 0 {
                                state.game.inventory_cursor = current - 1;
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Right => {
                            if current + 1 < inv_count {
                                state.game.inventory_cursor = current + 1;
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(item_id) =
                                state.game.inventory.get(state.game.inventory_cursor)
                            {
                                state.ui.active_item_popup = Some(item_id.clone());
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
