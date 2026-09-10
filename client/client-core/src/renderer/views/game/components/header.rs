use crate::events::{ApplicationEvent, SendEvent};
use crate::renderer::components::{Component, EventFlow, LabelButton, Lifecycle};
use crate::renderer::text::wrap_str_to_lines;
use crate::renderer::theme::{
    ERROR_COLOR, PLAYER_COLOR, ROOM_COLOR, SUCCESS_COLOR, WARNING_COLOR, default_block,
};
use crate::states::AppState;
use crate::states::game::{ChatState, HelpState, Overlay};
use client_api::ApiRequest;
use client_api::commands::{
    GroupCreateCommand, GroupLeaveCommand, QuitCommand, StatusCommand, WhoCommand,
};
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
    who: LabelButton,
    status: LabelButton,
    quit: LabelButton,
    group_create: LabelButton,
    group_leave: LabelButton,
    chat: LabelButton,
    help: LabelButton,
    trace: LabelButton,
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

impl Header {
    pub fn new() -> Self {
        Self {
            who: LabelButton::new("WHO"),
            status: LabelButton::new("STATUS"),
            quit: LabelButton::new("QUIT"),
            group_create: LabelButton::new("CREATE GROUP"),
            group_leave: LabelButton::new("LEAVE GROUP"),
            chat: LabelButton::new("CHAT"),
            help: LabelButton::new("HELP"),
            trace: LabelButton::new("TRACE"),
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

        let buttons = [
            group,
            &mut self.who,
            &mut self.status,
            &mut self.chat,
            &mut self.help,
            &mut self.trace,
            &mut self.quit,
        ];

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
                Style::default().fg(ROOM_COLOR).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]);

        let hp_color = if state.game.player.max_hp == 0 {
            Color::Reset
        } else {
            let percentage =
                (state.game.player.hp as f32 / state.game.player.max_hp as f32) * 100.0;
            if percentage > 50.0 {
                SUCCESS_COLOR
            } else if percentage > 25.0 {
                WARNING_COLOR
            } else {
                ERROR_COLOR
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
                    .fg(PLAYER_COLOR)
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
                        .fg(PLAYER_COLOR)
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
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let CrosstermEvent::Mouse(mouse) = event else {
            return EventFlow::Ignored;
        };

        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return EventFlow::Ignored;
        }

        if self.chat.hit(mouse.column, mouse.row) {
            state.game.overlays.toggle(Overlay::Chat(ChatState));
            return EventFlow::Consumed;
        }

        if self.help.hit(mouse.column, mouse.row) {
            state.game.overlays.toggle(Overlay::Help(HelpState));
            return EventFlow::Consumed;
        }

        if self.trace.hit(mouse.column, mouse.row) {
            state.ui.show_trace_log = !state.ui.show_trace_log;
            return EventFlow::Consumed;
        }

        let requests = [
            (&self.who, ApiRequest::Who(WhoCommand)),
            (&self.status, ApiRequest::Status(StatusCommand)),
            (&self.quit, ApiRequest::Quit(QuitCommand)),
            (
                &self.group_create,
                ApiRequest::GroupCreate(GroupCreateCommand),
            ),
            (&self.group_leave, ApiRequest::GroupLeave(GroupLeaveCommand)),
        ];

        let Some(request) = requests
            .into_iter()
            .find_map(|(button, request)| button.hit(mouse.column, mouse.row).then_some(request))
        else {
            return EventFlow::Ignored;
        };

        let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(request)));

        EventFlow::Consumed
    }
}
