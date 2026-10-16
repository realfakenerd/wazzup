use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Tracks active background chat history synchronization tasks by JID
/// to prevent redundant, concurrent backfill subprocesses.
#[derive(Clone, Default)]
pub struct InFlightSyncTracker {
    in_flight: Arc<Mutex<HashSet<String>>>,
}

impl InFlightSyncTracker {
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Attempts to acquire an in-flight sync lock for the given `jid`.
    /// Returns `true` if acquired, or `false` if already in-flight.
    pub fn acquire(&self, jid: &str) -> bool {
        let mut set = self.in_flight.lock().unwrap();
        set.insert(jid.to_string())
    }

    /// Releases the in-flight sync lock for the given `jid`.
    pub fn release(&self, jid: &str) {
        let mut set = self.in_flight.lock().unwrap();
        set.remove(jid);
    }

    /// Checks if a sync is currently in flight for the given `jid`.
    pub fn is_in_flight(&self, jid: &str) -> bool {
        let set = self.in_flight.lock().unwrap();
        set.contains(jid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_tracker_acquire_and_release() {
        let tracker = InFlightSyncTracker::new();
        let jid = "user1@s.whatsapp.net";

        assert!(!tracker.is_in_flight(jid));
        // First acquire succeeds
        assert!(tracker.acquire(jid));
        assert!(tracker.is_in_flight(jid));

        // Duplicate acquire while in flight is rejected
        assert!(!tracker.acquire(jid));

        // Release clears the state
        tracker.release(jid);
        assert!(!tracker.is_in_flight(jid));

        // Re-acquire after release succeeds
        assert!(tracker.acquire(jid));
    }

    #[test]
    fn test_sync_tracker_independent_jids() {
        let tracker = InFlightSyncTracker::new();
        let jid1 = "user1@s.whatsapp.net";
        let jid2 = "user2@s.whatsapp.net";

        assert!(tracker.acquire(jid1));
        assert!(tracker.acquire(jid2));

        assert!(tracker.is_in_flight(jid1));
        assert!(tracker.is_in_flight(jid2));

        tracker.release(jid1);
        assert!(!tracker.is_in_flight(jid1));
        assert!(tracker.is_in_flight(jid2));
    }
}
