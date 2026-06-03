use std::time::Duration;

pub const TICK_RATE: u16 = 48;
pub const TICK_TIME: Duration = Duration::from_millis(1000 / TICK_RATE as u64);


pub enum TickResult {
    TickEnd,
    Exit,
}