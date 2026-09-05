use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationTopic {
    Connection,
    Fight,
    GameServer,
    Protocol,
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationKind {
    Information,
    Warning,
    Error,
    Success,
}

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

const NOTIF_DEFAULT_DURATION: NotificationDuration =
    NotificationDuration::Finite(Duration::from_millis(5000));

pub struct Notification {
    pub message: String,
    pub kind: NotificationKind,
    pub topic: Option<NotificationTopic>,
    pub duration: NotificationDuration,
    pub created_at: Instant,
    pub paused_at: Option<Instant>,
    pub paused_duration: Duration,
}

impl Notification {
    pub fn new(message: String, kind: NotificationKind, duration: NotificationDuration) -> Self {
        Self {
            message,
            kind,
            topic: None,
            created_at: Instant::now(),
            duration,
            paused_at: None,
            paused_duration: Duration::ZERO,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(
            message.into(),
            NotificationKind::Information,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(
            message.into(),
            NotificationKind::Warning,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(
            message.into(),
            NotificationKind::Success,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(
            message.into(),
            NotificationKind::Error,
            NOTIF_DEFAULT_DURATION,
        )
    }

    pub fn with_topic(mut self, topic: NotificationTopic) -> Self {
        self.topic = Some(topic);
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
pub struct Notifications {
    entries: Vec<(NotificationId, Notification)>,
    next_id: u64,
}

impl Notifications {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_mut(&mut self, id: NotificationId) -> Option<&mut Notification> {
        self.entries
            .iter_mut()
            .find(|(entry_id, _)| *entry_id == id)
            .map(|(_, entry)| entry)
    }

    pub fn latest(&self, count: usize) -> Vec<(NotificationId, &Notification)> {
        self.entries
            .iter()
            .rev()
            .take(count)
            .map(|(id, entry)| (*id, entry))
            .collect()
    }

    pub fn push(&mut self, notification: Notification) {
        if let Some(topic) = notification.topic {
            self.entries.retain(|(_, entry)| entry.topic != Some(topic));
        }

        self.next_id += 1;
        self.entries
            .push((NotificationId(self.next_id), notification));
    }

    pub fn remove(&mut self, id: NotificationId) {
        self.entries.retain(|(entry_id, _)| *entry_id != id);
    }

    pub fn retain_active(&mut self) {
        self.entries.retain(|(_, entry)| match entry.duration {
            NotificationDuration::Infinite => true,
            NotificationDuration::Finite(total) => entry.elapsed() < total,
        });
    }
}
