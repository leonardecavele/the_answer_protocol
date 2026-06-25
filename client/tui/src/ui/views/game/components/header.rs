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

        let mut stats_text = format!(
            " HP: {}/{} | Online: {} ",
            state.game.hp, state.game.max_hp, state.game.online_players_count
        );

        if let Some(group_id) = &state.game.group_id {
            if let Some(leader_name) = &state.game.group_leader {
                let display_leader = if Some(leader_name) == state.game.player_name.as_ref() {
                    "[You]"
                } else {
                    leader_name.as_str()
                };
                stats_text = format!(
                    " Group: {} | Leader: {} | HP: {}/{} | Online: {} ",
                    group_id, display_leader, state.game.hp, state.game.max_hp, state.game.online_players_count
                );
            }
        }

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
