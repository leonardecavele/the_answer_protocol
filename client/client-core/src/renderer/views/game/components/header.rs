use crate::events::ApplicationEvent;
use crate::renderer::components::{CommandButton, Component, EventFlow, Lifecycle};
use crate::renderer::text::wrap_str_to_lines;
use crate::renderer::theme::default_block;
use crate::states::app::AppState;
use crossterm::event::{Event as CrosstermEvent, MouseButton, MouseEventKind};
use ratatui::widgets::Paragraph;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use tokio::sync::mpsc::Sender;

pub struct Header {
    who: CommandButton,
    status: CommandButton,
    quit: CommandButton,
    group_create: CommandButton,
    group_leave: CommandButton,
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

impl Header {
    pub fn new() -> Self {
        Self {
            who: CommandButton::new("WHO", "WHO"),
            status: CommandButton::new("STATUS", "STATUS"),
            quit: CommandButton::new("QUIT", "QUIT"),
            group_create: CommandButton::new("CREATE GROUP", "GROUP CREATE"),
            group_leave: CommandButton::new("LEAVE GROUP", "GROUP LEAVE"),
        }
    }

    fn draw_buttons(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let group = if state.game.group.is_in_group() {
            self.group_create.hide();
            &mut self.group_leave
        } else {
            self.group_leave.hide();
            &mut self.group_create
        };

        let buttons = [&mut self.who, &mut self.status, &mut self.quit, group];

        let mut x = area.x + 1;
        let y = area.bottom().saturating_sub(1);

        for button in buttons {
            let width = button.width();

            if x + width >= area.right() {
                button.hide();
                continue;
            }

            button.draw(frame, Rect::new(x, y, width, 1));
            x += width;
        }
    }
}

impl Component for Header {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let room = state.game.room.as_ref();

        let room_name = match room {
            Some(room) => room.name.as_str(),
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

        let description = match room {
            Some(room) => room.description.as_str(),
            None => "",
        };

        let inner_area = block.inner(area);
        let visual_lines = wrap_str_to_lines(description, inner_area.width as usize);

        let paragraph = Paragraph::new(visual_lines).block(block);

        frame.render_widget(paragraph, area);

        self.draw_buttons(state, frame, area);
    }
}

impl Lifecycle for Header {
    fn handle_device_event(
        &mut self,
        _state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let CrosstermEvent::Mouse(mouse) = event else {
            return EventFlow::Ignored;
        };

        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return EventFlow::Ignored;
        }

        let buttons = [
            &self.who,
            &self.status,
            &self.quit,
            &self.group_create,
            &self.group_leave,
        ];

        let Some(command) = buttons
            .iter()
            .find_map(|button| button.hit(mouse.column, mouse.row))
        else {
            return EventFlow::Ignored;
        };

        let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(command.to_string()));

        EventFlow::Consumed
    }
}
