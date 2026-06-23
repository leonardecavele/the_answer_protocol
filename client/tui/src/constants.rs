use std::time::Duration;

pub const TICK_RATE: Duration = Duration::from_millis(500);
pub const MAX_EVENTS_BUS: usize = 100;
pub const MAX_EVENT_HISTORY: usize = 100;
pub const MAX_VISIBLE_NOTIFICATIONS: usize = 5;