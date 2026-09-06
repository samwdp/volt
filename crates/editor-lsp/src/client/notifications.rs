use super::types::*;

impl LspClientManager {
    /// Returns a snapshot of recent UI-facing notifications emitted by the LSP client.
    pub fn notification_snapshot(&self) -> LspNotificationSnapshot {
        self.notifications
            .lock()
            .map(|log| log.snapshot())
            .unwrap_or_default()
    }

    /// Notification log revision without cloning entries.
    pub fn notification_revision(&self) -> u64 {
        self.notifications
            .lock()
            .map(|log| log.revision())
            .unwrap_or(0)
    }

    /// Clones UI notifications only when their revision moved.
    pub fn notification_snapshot_if_changed(
        &self,
        applied_revision: u64,
    ) -> Option<LspNotificationSnapshot> {
        let Ok(log) = self.notifications.lock() else {
            return None;
        };
        (log.revision() != applied_revision).then(|| log.snapshot())
    }
}
