use crate::events::NotificationType;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(PartialEq, Eq)]
pub enum NotificationDuration {
    Infinite,
    Finite(Duration),
}

impl NotificationDuration {
    pub fn from_ms(ms: u64) -> Self {
        Self::Finite(Duration::from_millis(ms))
    }
}

pub const NOTIF_DEFAULT_DURATION: NotificationDuration =
    NotificationDuration::Finite(Duration::from_millis(5000));

pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: NotificationDuration,
    pub created_at: Instant,
    pub paused_at: Option<Instant>,
    pub paused_duration: Duration,
}

impl Notification {
    pub fn new(
        id_opt: Option<String>,
        message: String,
        notif_type: NotificationType,
        duration: NotificationDuration,
    ) -> Self {
        Self {
            id: id_opt.unwrap_or_else(|| Uuid::new_v4().to_string()),
            message,
            notification_type: notif_type,
            created_at: Instant::now(),
            duration,
            paused_at: None,
            paused_duration: Duration::ZERO,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Information,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Warning,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Success,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Error,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_duration(mut self, duration: NotificationDuration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_ms(mut self, ms: u64) -> Self {
        self.duration = NotificationDuration::from_ms(ms);
        self
    }

    pub fn pause(&mut self) {
        if self.paused_at.is_none() {
            self.paused_at = Some(Instant::now());
        }
    }

    pub fn resume(&mut self) {
        if let Some(paused_at) = self.paused_at.take() {
            self.paused_duration += paused_at.elapsed();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        let now = Instant::now();

        let paused = self.paused_duration
            + self
                .paused_at
                .map(|paused_at| now.duration_since(paused_at))
                .unwrap_or(Duration::ZERO);

        self.created_at.elapsed().saturating_sub(paused)
    }

    pub fn remaining(&self) -> Option<Duration> {
        match self.duration {
            NotificationDuration::Infinite => None,
            NotificationDuration::Finite(total) => Some(total.saturating_sub(self.elapsed())),
        }
    }

    pub fn is_infinite(&self) -> bool {
        self.duration == NotificationDuration::Infinite
    }

    pub fn remaining_percent(&self) -> f32 {
        match self.duration {
            NotificationDuration::Infinite => 100.,
            NotificationDuration::Finite(duration) => {
                if duration.is_zero() {
                    return 0.0;
                }

                let elapsed = self.elapsed().as_secs_f32();
                let total = duration.as_secs_f32();

                100.0 - ((elapsed / total) * 100.0).clamp(0.0, 100.0)
            }
        }
    }
}

#[derive(Default)]
pub struct Notifications(Vec<Notification>);

impl Notifications {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, id: &str) -> Option<&Notification> {
        self.0.iter().find(|n| n.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Notification> {
        self.0.iter_mut().find(|n| n.id == id)
    }

    pub fn latest(&self, n: usize) -> Vec<&Notification> {
        self.0.iter().rev().take(n).collect::<Vec<_>>()
    }

    pub fn push(&mut self, notification: Notification) {
        self.0.push(notification);
    }

    pub fn remove(&mut self, target_id: &str) {
        self.0.retain(|n| n.id != target_id);
    }

    pub fn retain_active(&mut self) {
        self.0.retain(|n| match n.duration {
            NotificationDuration::Infinite => true,
            NotificationDuration::Finite(total) => n.elapsed() < total,
        });
    }
}
