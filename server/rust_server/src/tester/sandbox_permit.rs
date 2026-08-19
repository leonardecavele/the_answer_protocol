use std::sync::{Condvar, Mutex};

pub(super) const MAX_CONCURRENT_SANDBOXES: usize = 2;

static SANDBOX_QUEUE: (Mutex<usize>, Condvar) = (Mutex::new(0), Condvar::new());

pub(super) struct SandboxPermit;

impl SandboxPermit {
    pub(super) fn acquire() -> Self {
        let (active_sandboxes, slot_available) = &SANDBOX_QUEUE;
        let active_sandboxes = active_sandboxes
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut active_sandboxes = slot_available
            .wait_while(active_sandboxes, |active| {
                *active >= MAX_CONCURRENT_SANDBOXES
            })
            .unwrap_or_else(|error| error.into_inner());

        *active_sandboxes += 1;
        Self
    }
}

impl Drop for SandboxPermit {
    fn drop(&mut self) {
        let (active_sandboxes, slot_available) = &SANDBOX_QUEUE;
        let mut active_sandboxes = active_sandboxes
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        *active_sandboxes -= 1;
        drop(active_sandboxes);
        slot_available.notify_one();
    }
}
