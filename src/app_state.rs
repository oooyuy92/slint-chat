use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{assistant::Assistant, message::Message, topic::Topic};

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub assistants: Vec<Assistant>,
    pub topics: Vec<Topic>,
    pub active_topic_id: Option<String>,
    pub messages: HashMap<String, Vec<Message>>,
}

impl AppState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn active_messages(&self) -> &[Message] {
        self.active_topic_id
            .as_ref()
            .and_then(|id| self.messages.get(id))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn push_message(&mut self, msg: Message) {
        self.messages
            .entry(msg.topic_id.clone())
            .or_default()
            .push(msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::message::Role;

    #[test]
    fn test_push_and_retrieve() {
        let state = AppState::new();
        let mut s = state.lock().unwrap();
        s.active_topic_id = Some("t1".into());
        s.push_message(Message {
            id: "m1".into(),
            topic_id: "t1".into(),
            role: Role::User,
            content: "hi".into(),
            created_at: 0,
        });
        assert_eq!(s.active_messages().len(), 1);
    }
}
