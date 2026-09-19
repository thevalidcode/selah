//! Presentation state manipulators.
//!
//! The state struct itself lives in `models::presentation` so it can be
//! serialized and shared; the state-transition logic lives here.

use crate::models::presentation::{PresentationItem, PresentationState};

impl PresentationState {
    pub fn empty() -> Self {
        Self {
            current: None,
            queue: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Sets the current (projected) item, pushing the previous one onto the
    /// history stack.
    pub fn set_current(&mut self, item: PresentationItem) {
        if let Some(previous) = self.current.take() {
            self.history.push(previous);
        }
        self.current = Some(item);
    }

    pub fn clear(&mut self) {
        if let Some(previous) = self.current.take() {
            self.history.push(previous);
        }
    }

    /// Shows the next queued item, if any.
    pub fn show_next(&mut self) -> bool {
        if self.queue.is_empty() {
            return false;
        }
        let next = self.queue.remove(0);
        self.set_current(next);
        true
    }

    /// Goes back to the most recent item in history, if any.
    pub fn show_previous(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        // The currently projected item, when present, moves to the front of
        // the queue so it can be returned to with Next.
        if let Some(current) = self.current.take() {
            self.queue.insert(0, current);
        }
        self.current = Some(previous);
        true
    }

    pub fn current(&self) -> Option<&PresentationItem> {
        self.current.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::presentation::ContentPayload;

    fn item(title: &str) -> PresentationItem {
        PresentationItem {
            id: title.to_string(),
            content_type: crate::models::presentation::ContentType::Text,
            title: title.to_string(),
            payload: ContentPayload::Text {
                heading: Some(title.to_string()),
                text: title.to_string(),
            },
        }
    }

    #[test]
    fn set_current_keeps_history() {
        let mut state = PresentationState::empty();
        state.set_current(item("first"));
        state.set_current(item("second"));
        assert_eq!(state.current().unwrap().title, "second");
        assert_eq!(state.history.len(), 1);
    }

    #[test]
    fn next_and_previous_roundtrip() {
        let mut state = PresentationState::empty();
        state.set_current(item("a"));
        state.set_current(item("b"));
        state.queue.push(item("c"));

        assert!(state.show_next());
        assert_eq!(state.current().unwrap().title, "c");
        assert!(state.show_previous());
        assert_eq!(state.current().unwrap().title, "b");
    }

    #[test]
    fn clear_moves_to_history() {
        let mut state = PresentationState::empty();
        state.set_current(item("a"));
        state.clear();
        assert!(state.current().is_none());
        assert_eq!(state.history.len(), 1);
    }
}
