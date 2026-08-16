use crate::data::manifest::Manifest;
use api_client::commands::{FightAttackStatus, QuestData};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

//region Dialogue
pub const END_OF_DIALOGUE_TAG: &str = "[end of dialogue]";

#[derive(Debug, Clone)]
pub struct DialogueState {
    pub npc_id: String,
    pub npc_name: String,
    pub full_text: String,
    pub visible_chars: usize,
    pub ends_dialog: bool,
    pub last_tick: Instant,
}

impl DialogueState {
    pub fn new(npc_id: String, npc_name: String, full_text: String, ends_dialog: bool) -> Self {
        Self {
            npc_id,
            npc_name,
            full_text,
            visible_chars: 0,
            ends_dialog,
            last_tick: Instant::now(),
        }
    }

    pub fn add(&mut self, text: String, ends_dialog: bool) {
        self.full_text.push_str("\n\n");
        self.full_text.push_str(&text);
        self.ends_dialog = ends_dialog;
        self.visible_chars += 2;
    }
}
//endregion

//region Overlay
pub enum Overlay {
    Help,
    Chat,
    NpcActions { npc_id: String },
    ItemActions { item_id: String },
    ItemView { item_id: String },
    QuestView { quest_id: String },
    Dialogue(DialogueState),
}

#[derive(Clone, Copy, PartialEq)]
pub enum OverlayKind {
    Help,
    Chat,
    NpcActions,
    ItemActions,
    ItemView,
    QuestView,
    Dialogue,
}

impl Overlay {
    pub fn kind(&self) -> OverlayKind {
        match self {
            Overlay::Help => OverlayKind::Help,
            Overlay::Chat => OverlayKind::Chat,
            Overlay::NpcActions { .. } => OverlayKind::NpcActions,
            Overlay::ItemActions { .. } => OverlayKind::ItemActions,
            Overlay::ItemView { .. } => OverlayKind::ItemView,
            Overlay::QuestView { .. } => OverlayKind::QuestView,
            Overlay::Dialogue(_) => OverlayKind::Dialogue,
        }
    }
}

impl OverlayKind {
    pub fn is_modal(&self) -> bool {
        !matches!(self, OverlayKind::Chat)
    }
}
//endregion

//region Chat
#[derive(Debug, Clone)]
pub enum ChatChannel {
    Global,
    Private(String),
    Room,
    Group,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub sender: String,
    pub content: String,
}
//endregion

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameFocus {
    Input,
    RightPanel,
    NpcList,
    QuestList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl Default for GameFocus {
    fn default() -> Self {
        GameFocus::Input
    }
}

//region Sub-states
pub struct PlayerState {
    pub name: Option<String>,
    pub hp: u32,
    pub max_hp: u32,
    pub inventory: Vec<String>,
    pub quests: Vec<QuestData>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: None,
            hp: 100,
            max_hp: 100,
            inventory: Vec::new(),
            quests: Vec::new(),
        }
    }

    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
    }

    pub fn is_dead(&self) -> bool {
        self.hp == 0
    }
}

pub struct GroupState {
    pub id: Option<String>,
    pub leader: Option<String>,
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            id: None,
            leader: None,
        }
    }

    pub fn is_in_group(&self) -> bool {
        self.id.is_some()
    }

    pub fn is_leader(&self, player_name: &str) -> bool {
        if let Some(leader) = &self.leader {
            leader == player_name
        } else {
            false
        }
    }

    pub fn leave(&mut self) {
        self.id = None;
        self.leader = None;
    }
}

pub struct RoomState {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub exits: HashMap<String, String>,
    pub players: Vec<String>,
    pub npcs: Vec<String>,
    pub items: Vec<String>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            exits: HashMap::new(),
            players: Vec::new(),
            npcs: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.id = None;
        self.name = None;
        self.description = None;
        self.exits.clear();
        self.players.clear();
        self.npcs.clear();
        self.items.clear();
    }

    pub fn has_exit(&self, direction: &str) -> bool {
        self.exits.contains_key(direction)
    }
}

pub struct ServerState {
    pub online_players_count: u32,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            online_players_count: 1,
        }
    }
}

pub struct GameUiState {
    pub current_focus: GameFocus,
    pub inventory_cursor: usize,
    pub focused_entity_id: Option<String>,
    pub dialogue_closed_at: Option<Instant>,
    overlays: Vec<Overlay>,
}

impl GameUiState {
    pub fn new() -> Self {
        Self {
            current_focus: GameFocus::default(),
            inventory_cursor: 0,
            focused_entity_id: None,
            dialogue_closed_at: None,
            overlays: Vec::new(),
        }
    }

    pub fn open(&mut self, overlay: Overlay) {
        let kind = overlay.kind();
        self.overlays.retain(|o| o.kind() != kind);
        self.overlays.push(overlay);
    }

    pub fn toggle(&mut self, overlay: Overlay) {
        let kind = overlay.kind();
        if self.is_open(kind) {
            self.close(kind);
        } else {
            self.open(overlay);
        }
    }

    pub fn close_top(&mut self) {
        if let Some(overlay) = self.overlays.pop() {
            self.after_close(overlay.kind());
        }
    }

    pub fn close(&mut self, kind: OverlayKind) {
        self.overlays.retain(|o| o.kind() != kind);
        self.after_close(kind);
    }

    pub fn close_all(&mut self) {
        let had_dialogue = self.is_open(OverlayKind::Dialogue);
        self.overlays.clear();
        if had_dialogue {
            self.after_close(OverlayKind::Dialogue);
        }
    }

    pub fn overlays(&self) -> impl Iterator<Item = &Overlay> {
        self.overlays.iter()
    }

    pub fn top_kind(&self) -> Option<OverlayKind> {
        self.overlays.last().map(Overlay::kind)
    }

    pub fn is_open(&self, kind: OverlayKind) -> bool {
        self.overlays.iter().any(|o| o.kind() == kind)
    }

    pub fn target_of(&self, kind: OverlayKind) -> Option<&str> {
        self.overlays
            .iter()
            .rev()
            .find(|o| o.kind() == kind)
            .and_then(|o| match o {
                Overlay::NpcActions { npc_id } => Some(npc_id.as_str()),
                Overlay::ItemActions { item_id } | Overlay::ItemView { item_id } => {
                    Some(item_id.as_str())
                }
                Overlay::QuestView { quest_id } => Some(quest_id.as_str()),
                _ => None,
            })
    }

    pub fn dialogue(&self) -> Option<&DialogueState> {
        self.overlays.iter().rev().find_map(|o| match o {
            Overlay::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
    }

    pub fn dialogue_mut(&mut self) -> Option<&mut DialogueState> {
        self.overlays.iter_mut().rev().find_map(|o| match o {
            Overlay::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
    }

    pub fn is_npc_dialogue_available(&self) -> bool {
        if let Some(time) = self.dialogue_closed_at {
            if time.elapsed() < Duration::from_millis(300) {
                return false;
            }
        }
        true
    }

    fn after_close(&mut self, kind: OverlayKind) {
        if kind == OverlayKind::Dialogue {
            self.focused_entity_id = None;
            self.dialogue_closed_at = Some(Instant::now());
        }
    }
}
//endregion

//region Fight
pub struct FightState {
    pub submitted: bool,
    pub status: Option<FightAttackStatus>,
}

impl FightState {
    pub fn new() -> Self {
        Self {
            submitted: false,
            status: None,
        }
    }

    pub fn reset(&mut self) {
        self.submitted = false;
        self.status = None;
    }
}
//endregion

pub struct GameState {
    pub player: PlayerState,
    pub group: GroupState,
    pub room: RoomState,
    pub server: ServerState,
    pub ui: GameUiState,
    pub fight: FightState,
    pub manifest: Arc<Manifest>,

    pub chat_history: Vec<ChatMessage>,
    pub action_logs: Vec<String>,
}

impl GameState {
    pub fn new(manifest: Arc<Manifest>) -> Self {
        Self {
            player: PlayerState::new(),
            group: GroupState::new(),
            room: RoomState::new(),
            server: ServerState::new(),
            ui: GameUiState::new(),
            fight: FightState::new(),
            manifest,
            chat_history: Vec::new(),
            action_logs: Vec::new(),
        }
    }

    pub fn log_action(&mut self, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.action_logs.push(format!("[{}] {}", time_str, text));
    }
}
