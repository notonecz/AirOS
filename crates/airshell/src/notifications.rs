// crates/airshell/src/notifications.rs

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct NotificationEntry {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

/// Fronta notifikací s omezenou kapacitou (FIFO, oldest first).
pub struct NotificationQueue {
    entries: VecDeque<NotificationEntry>,
    max_size: usize,
    next_id: u64,
}

impl NotificationQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size,
            next_id: 1,
        }
    }

    /// Přidá notifikaci a vrátí její ID. Překročení max_size odstraní nejstarší.
    pub fn push(&mut self, title: String, body: String, icon: Option<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push_back(NotificationEntry {
            id,
            title,
            body,
            icon,
        });
        if self.entries.len() > self.max_size {
            self.entries.pop_front();
        }
        id
    }

    /// Odstraní notifikaci dle ID. Vrátí true pokud existovala.
    pub fn dismiss(&mut self, id: u64) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < before
    }

    pub fn entries(&self) -> &VecDeque<NotificationEntry> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let q = NotificationQueue::new(10);
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn push_returns_incrementing_ids() {
        let mut q = NotificationQueue::new(10);
        let id1 = q.push("A".into(), "body".into(), None);
        let id2 = q.push("B".into(), "body".into(), None);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn push_beyond_max_drops_oldest() {
        let mut q = NotificationQueue::new(2);
        let id1 = q.push("A".into(), "b".into(), None);
        q.push("B".into(), "b".into(), None);
        q.push("C".into(), "b".into(), None);
        assert_eq!(q.len(), 2);
        assert!(q.entries().iter().all(|e| e.id != id1));
    }

    #[test]
    fn dismiss_removes_entry() {
        let mut q = NotificationQueue::new(10);
        let id = q.push("Hello".into(), "World".into(), None);
        let removed = q.dismiss(id);
        assert!(removed);
        assert!(q.is_empty());
    }

    #[test]
    fn dismiss_nonexistent_returns_false() {
        let mut q = NotificationQueue::new(10);
        assert!(!q.dismiss(999));
    }
}
