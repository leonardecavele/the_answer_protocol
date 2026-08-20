use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::theme::default_block;
use crate::ui::utils::wrap_str_to_lines;
use ratatui::widgets::Paragraph;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub struct Header;

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

impl Header {
    pub fn new() -> Self {
        Self
    }
}

impl Component for Header {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let room_name = match &state.game.room.name {
            Some(name) => name.as_str(),
            None => "Cluster 6 (the backrooms)",
        };

        let title_line = Line::from(vec![
            Span::styled(" Room: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                room_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]);

        let hp_color = if state.game.player.max_hp == 0 {
            Color::Reset
        } else {
            let percentage =
                (state.game.player.hp as f32 / state.game.player.max_hp as f32) * 100.0;
            if percentage > 50.0 {
                Color::Green
            } else if percentage > 25.0 {
                Color::Yellow
            } else {
                Color::Red
            }
        };

        let stats_line = Line::from(vec![
            Span::styled(" Player: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                state
                    .game
                    .player
                    .name
                    .clone()
                    .unwrap_or("unknown".to_string()),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | HP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}/{}", state.game.player.hp, state.game.player.max_hp),
                Style::default().fg(hp_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " | Online: {} ",
                state.game.server.online_players_count
            )),
        ]);

        let mut block = default_block()
            .title(title_line.alignment(Alignment::Left))
            .title(stats_line.alignment(Alignment::Right));

        if let Some(group_id) = &state.game.group.id
            && let Some(leader_name) = &state.game.group.leader
        {
            let display_leader = if Some(leader_name) == state.game.player.name.as_ref() {
                "[You]"
            } else {
                leader_name.as_str()
            };

            let short_id = if group_id.len() > 8 {
                format!("{}...", &group_id[..8])
            } else {
                group_id.clone()
            };

            let group_line = Line::from(vec![
                Span::raw(format!(" Group: {} | Leader: ", short_id)),
                Span::styled(
                    display_leader,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]);

            block = block.title_bottom(group_line.alignment(Alignment::Right));
        }

        let description = match &state.game.room.description {
            Some(desc) => desc.as_str(),
            None => "",
        };

        let inner_area = block.inner(area);
        let visual_lines = wrap_str_to_lines(description, inner_area.width as usize);

        let paragraph = Paragraph::new(visual_lines).block(block);

        frame.render_widget(paragraph, area);
    }
}

impl Lifecycle for Header {}
