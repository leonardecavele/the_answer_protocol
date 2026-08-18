use crate::data::manifest::Manifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const CLOCKWISE: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub fn quarter_turns(self) -> usize {
        match self {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        }
    }

    pub fn from_quarter_turns(turns: usize) -> Direction {
        Direction::CLOCKWISE[turns % 4]
    }

    pub fn key(self) -> &'static str {
        match self {
            Direction::North => "NORTH",
            Direction::East => "EAST",
            Direction::South => "SOUTH",
            Direction::West => "WEST",
        }
    }

    fn from_code(code: char) -> Option<Direction> {
        match code.to_ascii_uppercase() {
            'N' => Some(Direction::North),
            'E' => Some(Direction::East),
            'S' => Some(Direction::South),
            'W' => Some(Direction::West),
            _ => None,
        }
    }

    pub fn facing_of_room(id: &str, manifest: &Manifest) -> Direction {
        manifest
            .rooms
            .get(id)
            .and_then(|room_entry| room_entry.direction)
            .and_then(Direction::from_code)
            .unwrap_or_default()
    }
}
