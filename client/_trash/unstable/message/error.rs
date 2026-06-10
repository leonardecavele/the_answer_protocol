#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub code: u16,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NameInUse,
    NoExit,

    NotInGroup,
    AlreadyInGroup,

    ItemNotFound,
    ItemNotInInventory,
    NpcNotFound,

    NpcNotHostile,
    NoQuestAvailable,

    ConnectionFailed,
    SendFailed,
}

impl Error {
    pub const NAME_IN_USE: Self = Self {
        code: 201,
        kind: ErrorKind::NameInUse,
    };

    pub const NO_EXIT: Self = Self {
        code: 301,
        kind: ErrorKind::NoExit,
    };

    pub const NOT_IN_GROUP: Self = Self {
        code: 401,
        kind: ErrorKind::NotInGroup,
    };

    pub const ALREADY_IN_GROUP: Self = Self {
        code: 402,
        kind: ErrorKind::AlreadyInGroup,
    };

    pub const ITEM_NOT_FOUND: Self = Self {
        code: 404,
        kind: ErrorKind::ItemNotFound,
    };

    pub const ITEM_NOT_IN_INVENTORY: Self = Self {
        code: 404,
        kind: ErrorKind::ItemNotInInventory,
    };

    pub const NPC_NOT_FOUND: Self = Self {
        code: 404,
        kind: ErrorKind::NpcNotFound,
    };

    pub const NPC_NOT_HOSTILE: Self = Self {
        code: 405,
        kind: ErrorKind::NpcNotHostile,
    };

    pub const NO_QUEST_AVAILABLE: Self = Self {
        code: 406,
        kind: ErrorKind::NoQuestAvailable,
    };

    pub const CONNECTION_FAILED: Self = Self {
        code: 900,
        kind: ErrorKind::ConnectionFailed,
    };

    pub const SEND_FAILED: Self = Self {
        code: 901,
        kind: ErrorKind::SendFailed,
    };
}
