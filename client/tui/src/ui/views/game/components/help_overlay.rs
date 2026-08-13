use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::OverlayKind;
use crate::ui::components::Lifecycle;
use crate::ui::components::scrollable::ScrollableComponent;
use crate::ui::theme::overlay_block;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mpsc::Sender;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Block,
};
use tokio::sync::mpsc;

const HELP_WIDTH: u16 = 60;
const HELP_HEIGHT: u16 = 20;

pub struct HelpOverlayComponent;

impl HelpOverlayComponent {
    pub fn new() -> Self {
        Self
    }
}

impl ScrollableComponent for HelpOverlayComponent {
    fn get_area(&self, state: &AppState, max_area: Rect) -> Rect {
        if !state.game.ui.is_open(OverlayKind::Help) {
            return Rect::default();
        }

        let x = max_area.width.saturating_sub(HELP_WIDTH) / 2;
        let y = max_area.height.saturating_sub(HELP_HEIGHT) / 2;

        Rect {
            x: max_area.x + x,
            y: max_area.y + y,
            width: HELP_WIDTH,
            height: HELP_HEIGHT,
        }
    }

    fn get_block<'a>(&self, _state: &AppState) -> Block<'a> {
        overlay_block()
            .title(Line::from(" Keyboard Shortcuts ").alignment(Alignment::Center))
            .title_bottom(Line::from(" Press Ctrl+H to close ").alignment(Alignment::Center))
    }

    fn get_content<'a>(&self, _state: &'a AppState, _max_width: usize) -> Vec<Line<'a>> {
        vec![
            Line::from(vec![Span::styled(
                "Global",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ctrl+c: quit game"),
            Line::from("  ctrl+h: toggle help"),
            Line::from("  ctrl+e: toggle event overlay"),
            Line::from("  f1: toggle chat overlay"),
            Line::from("  mouse click: focus panels (input, room npcs, image)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Input panel",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  enter: send command / focus right panel"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Right panel (details/movement)",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  up/down/left/right: move north/south/west/east"),
            Line::from("  enter: focus room npcs list"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Room npcs list",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  up/down: select npc"),
            Line::from("  enter: open interaction menu"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Interaction menus & chat",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  esc: close menus"),
            Line::from("  up/down: change action (talk/attack) / scroll chat"),
            Line::from("  enter: execute action / next dialogue"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Text commands (input)",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  connect : connect to server"),
            Line::from("  quit : disconnect and close the game"),
            Line::from("  look : look around the room"),
            Line::from("  move <NORTH|SOUTH|EAST|WEST> : move to direction"),
            Line::from("  who : see online players"),
            Line::from("  say <msg> : send a global message"),
            Line::from("  msg <name> <msg> : send a private message"),
            Line::from("  take <item> : take an item"),
            Line::from("  drop <item> : drop an item"),
            Line::from("  inv : view inventory"),
            Line::from("  status : view status"),
            Line::from("  talk <npc> : talk to npc"),
            Line::from("  attack <npc> : attack npc"),
            Line::from("  quest : view active quest details"),
            Line::from("  quests : list all active quests"),
            Line::from("  group_create : create a new group"),
            Line::from("  group_join <name> : join a player's group"),
            Line::from("  group_invite <name> : invite a player to your group"),
            Line::from("  group_leave : leave your current group"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Hud & status",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  top right : your hp and max hp"),
            Line::from("  top right : online players count"),
            Line::from("  bottom right : notifications (disappear after 5s)"),
        ]
    }
}

impl Lifecycle for HelpOverlayComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    state.game.ui.close(OverlayKind::Help);
                    return true;
                }
                KeyCode::Char('h') => {
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        state.game.ui.close(OverlayKind::Help);
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}
