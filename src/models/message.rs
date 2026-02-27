use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub topic_id: String,
    pub role: Role,
    pub content: String,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_role_serde() {
        let json = serde_json::to_string(&Role::User).unwrap();
        assert_eq!(json, "\"user\"");
        let r: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(r, Role::User);
    }

    #[test]
    fn test_message_serde() {
        let m = Message {
            id: "m1".into(),
            topic_id: "t1".into(),
            role: Role::Assistant,
            content: "Hello".into(),
            created_at: 0,
        };
        let json = serde_json::to_string(&m).unwrap();
        let m2: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(m.id, m2.id);
        assert_eq!(m2.role, Role::Assistant);
    }
}
