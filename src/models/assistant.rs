use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assistant {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub default_model: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serde() {
        let a = Assistant {
            id: "1".into(),
            name: "Default".into(),
            system_prompt: "You are helpful.".into(),
            default_model: "gpt-4o".into(),
        };
        let json = serde_json::to_string(&a).unwrap();
        let a2: Assistant = serde_json::from_str(&json).unwrap();
        assert_eq!(a.id, a2.id);
    }
}
