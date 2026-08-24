use crate::collections::BoundedLog;
use crate::states::notification::Notifications;

pub struct UiState {
    pub notifications: Notifications,
    pub show_trace_log: bool,
    pub trace_log: BoundedLog<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    pub fn new() -> Self {
        Self {
            notifications: Notifications::default(),
            show_trace_log: false,
            trace_log: BoundedLog::with_max_capacity(100),
        }
    }
}
