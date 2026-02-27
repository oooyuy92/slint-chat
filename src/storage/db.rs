use anyhow::Result;
use rusqlite::{Connection, params};

use crate::models::{
    assistant::Assistant,
    message::{Message, Role},
    topic::Topic,
};
use crate::app_state::AppState;

pub struct Db(Connection);

impl Db {
    pub fn init(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS assistants (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                default_model TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS topics (
                id TEXT PRIMARY KEY,
                assistant_id TEXT NOT NULL,
                title TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                topic_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
        ")?;
        Ok(Self(conn))
    }

    pub fn save_assistant(&self, a: &Assistant) -> Result<()> {
        self.0.execute(
            "INSERT OR REPLACE INTO assistants (id, name, system_prompt, default_model) VALUES (?1, ?2, ?3, ?4)",
            params![a.id, a.name, a.system_prompt, a.default_model],
        )?;
        Ok(())
    }

    pub fn save_topic(&self, t: &Topic) -> Result<()> {
        self.0.execute(
            "INSERT OR REPLACE INTO topics (id, assistant_id, title, model, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![t.id, t.assistant_id, t.title, t.model, t.created_at],
        )?;
        Ok(())
    }

    pub fn save_message(&self, m: &Message) -> Result<()> {
        let role_str = match m.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
        };
        self.0.execute(
            "INSERT OR REPLACE INTO messages (id, topic_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![m.id, m.topic_id, role_str, m.content, m.created_at],
        )?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<AppState> {
        let mut state = AppState::default();

        let mut stmt = self.0.prepare("SELECT id, name, system_prompt, default_model FROM assistants")?;
        state.assistants = stmt.query_map([], |row| {
            Ok(Assistant {
                id: row.get(0)?,
                name: row.get(1)?,
                system_prompt: row.get(2)?,
                default_model: row.get(3)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        let mut stmt = self.0.prepare("SELECT id, assistant_id, title, model, created_at FROM topics")?;
        state.topics = stmt.query_map([], |row| {
            Ok(Topic {
                id: row.get(0)?,
                assistant_id: row.get(1)?,
                title: row.get(2)?,
                model: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        let mut stmt = self.0.prepare("SELECT id, topic_id, role, content, created_at FROM messages")?;
        let messages: Vec<Message> = stmt.query_map([], |row| {
            let role_str: String = row.get(2)?;
            let role = match role_str.as_str() {
                "assistant" => Role::Assistant,
                "system" => Role::System,
                _ => Role::User,
            };
            Ok(Message {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                role,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        for msg in messages {
            state.messages.entry(msg.topic_id.clone()).or_default().push(msg);
        }

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::message::Role;

    #[test]
    fn test_save_and_load() {
        let db = Db::init(":memory:").unwrap();

        let assistant = Assistant {
            id: "a1".into(),
            name: "Default".into(),
            system_prompt: "You are helpful.".into(),
            default_model: "gpt-4o".into(),
        };
        db.save_assistant(&assistant).unwrap();

        let topic = Topic {
            id: "t1".into(),
            assistant_id: "a1".into(),
            title: "Test Topic".into(),
            model: "gpt-4o".into(),
            created_at: 0,
        };
        db.save_topic(&topic).unwrap();

        let msg = Message {
            id: "m1".into(),
            topic_id: "t1".into(),
            role: Role::User,
            content: "Hello".into(),
            created_at: 0,
        };
        db.save_message(&msg).unwrap();

        let state = db.load_all().unwrap();
        assert_eq!(state.assistants.len(), 1);
        assert_eq!(state.topics.len(), 1);
        assert_eq!(state.messages.get("t1").unwrap().len(), 1);
        assert_eq!(state.messages["t1"][0].content, "Hello");
    }
}
