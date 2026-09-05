use crate::collections::{Step, move_index};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoginFocus {
    #[default]
    PlayerName,
    ServerIp,
    ServerPort,
    ConnectButton,
}

impl LoginFocus {
    pub const FOCUS_COUNT: usize = 4;

    const ORDER: [LoginFocus; Self::FOCUS_COUNT] = [
        LoginFocus::PlayerName,
        LoginFocus::ServerIp,
        LoginFocus::ServerPort,
        LoginFocus::ConnectButton,
    ];

    fn index(self) -> usize {
        match self {
            LoginFocus::PlayerName => 0,
            LoginFocus::ServerIp => 1,
            LoginFocus::ServerPort => 2,
            LoginFocus::ConnectButton => 3,
        }
    }

    pub fn next(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Next)]
    }

    pub fn prev(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Previous)]
    }
}
