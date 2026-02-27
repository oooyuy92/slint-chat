use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub assistant_id: String,
    pub title: String,
    pub model: String,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serde() {
        let t = Topic {
            id: "t1".into(),
            assistant_id: "a1".into(),
            title: "Test Topic".into(),
            model: "gpt-4o".into(),
            created_at: 0,
        };
        let json = serde_json::to_string(&t).unwrap();
        let t2: Topic = serde_json::from_str(&json).unwrap();
        assert_eq!(t.id, t2.id);
    }
}
