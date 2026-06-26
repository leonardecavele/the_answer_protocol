use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
};

pub struct CenterPanelComponent {
    pub inventory_cols: usize,
    pub inventory_area: Option<Rect>,
}

impl CenterPanelComponent {
    pub fn new() -> Self {
        Self {
            inventory_cols: 1,
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

        let history_area = chunks[0];
        let inventory_area = chunks[1];
        self.inventory_area = Some(inventory_area);

        // 1. Action History
        let history_block = crate::ui::theme::default_block()
            .title(" Action History ")
            .title_bottom(
                ratatui::text::Line::from(" Press Ctrl + H to open help ")
                    .alignment(ratatui::layout::Alignment::Center),
            );

        let inner_history_area = history_block.inner(history_area);
        let max_width = inner_history_area.width as usize;

        let visual_lines =
            crate::ui::utils::wrap_slice_to_lines(&state.game.action_logs, max_width);

        let logs_count = visual_lines.len() as u16;
        let inner_height = inner_history_area.height;

        let scroll = if logs_count > inner_height {
            logs_count - inner_height
        } else {
            0
        };

        let history_list = ratatui::widgets::Paragraph::new(visual_lines)
            .block(history_block)
            .scroll((scroll, 0));

        frame.render_widget(history_list, history_area);

        let mut inv_block = crate::ui::theme::default_block().title(" Inventory ");
        if state.ui.current_focus == crate::states::ui::GameFocus::InventoryGrid {
            inv_block = inv_block.border_style(Style::default().fg(Color::Yellow));
        }

        let inv_inner = inv_block.inner(inventory_area);
        frame.render_widget(inv_block, inventory_area);

        if state.game.inventory.is_empty() {
            let p = ratatui::widgets::Paragraph::new(" Your inventory is empty. ")
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(p, inv_inner);
            return;
        }

        let item_width = 20;
        let item_height = 10;
        let cols = (inv_inner.width / item_width).max(1) as usize;
        self.inventory_cols = cols;

        for (idx, item_id) in state.game.inventory.iter().enumerate() {
            let row = idx / cols;
            let col = idx % cols;

            let cell_x = inv_inner.x + (col as u16) * item_width;
            let cell_y = inv_inner.y + (row as u16) * item_height;

            if cell_y >= inv_inner.bottom() {
                break;
            }

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: item_width.min(inv_inner.right().saturating_sub(cell_x)),
                height: item_height.min(inv_inner.bottom().saturating_sub(cell_y)),
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
                    let image_widget = ratatui_image::StatefulImage::default()
                        .resize(ratatui_image::Resize::Fit(None));
                    frame.render_stateful_widget(image_widget, image_area, protocol);
                }
            }

            let text = format!("{}\n{}", display_name, item_id);
            let mut p_style = Style::default();
            if state.ui.current_focus == crate::states::ui::GameFocus::InventoryGrid
                && state.game.inventory_cursor == idx
            {
                p_style = p_style.add_modifier(Modifier::REVERSED).fg(Color::Yellow);
            }
            let paragraph = ratatui::widgets::Paragraph::new(text)
                .alignment(ratatui::layout::Alignment::Center)
                .style(p_style);
            frame.render_widget(paragraph, text_area);
        }
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        if state.ui.current_focus == crate::states::ui::GameFocus::InventoryGrid {
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
