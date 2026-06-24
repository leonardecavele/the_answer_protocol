use std::time::Duration;

pub const TICK_RATE: Duration = Duration::from_millis(500);

pub const MAX_EVENTS_BUS: usize = 100;
pub const MAX_EVENT_HISTORY: usize = 100;
pub const MAX_VISIBLE_NOTIFICATIONS: usize = 5;

pub const NOTIF_ID_CONNECTION_ATTEMPT: &str = "notif_connection_attempt";
pub const NOTIF_DEFAULT_DURATION_MS: u64 = 5000;

pub const ASSETS_PATH_MANIFEST: &str = "assets/manifest.json";
pub const DEFAULT_SERVER_IP: &str = "127.0.0.1";
pub const DEFAULT_SERVER_PORT: &str = "38800";
