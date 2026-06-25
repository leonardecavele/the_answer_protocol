use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

pub struct HeaderComponent;

impl HeaderComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for HeaderComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let title = match &state.game.current_room_name {
            Some(name) => format!(" {} ", name),
            None => " Cluster 6 (the backrooms) ".to_string(),
        };

        let stats_text = format!(
            " HP: {}/{} | Online: {} ",
            state.game.hp, state.game.max_hp, state.game.online_players_count
        );

        let description = match &state.game.current_room_description {
            Some(desc) => desc.as_str(),
            None => "",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(ratatui::text::Line::from(title).alignment(ratatui::layout::Alignment::Left))
            .title(
                ratatui::text::Line::from(stats_text).alignment(ratatui::layout::Alignment::Right),
            );

        let inner_area = block.inner(area);
        let visual_lines =
            crate::ui::utils::wrap_str_to_lines(description, inner_area.width as usize);

        let paragraph = ratatui::widgets::Paragraph::new(visual_lines).block(block);

        frame.render_widget(paragraph, area);
    }
}
