use serde::{Deserialize, Serialize};
use spin_sdk::{key_value::Store, http::conversions::IntoBody};

const SYSTEM_INSTRUCTION: &str = r#"<<SYS>>
You are an AI sidekick and you help users deepen their geographic knowledge.
Answer the questions as short as possible.
Never write any kind of introduction, greeting, or sign-off.
<</SYS>>"#;

#[derive(Debug, Deserialize, Serialize)]
pub struct Conversation {
    pub id: String,
    pub interactions: Vec<Interaction>,
}

impl Conversation {

    pub fn new (id: String) -> Self {
        Conversation {
            id,
            interactions: Vec::new(),
        }
    }

    pub fn exists(id: &str) -> bool {
        match Store::open_default() {
            Ok(store) => {
                store.exists(id).unwrap_or(false)
            }
            Err(_) => false,
        }
    }

    pub fn load(id: &str) -> anyhow::Result<Self> {
        let store = Store::open_default()?;
        match store.get_json::<Conversation>(id)? {
            Some(c) => Ok(c),
            None => Ok(Conversation::new(id.to_string()))
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let store = Store::open_default()?;
        store.set_json(&self.id, self)?;
        Ok(())
    }

    pub fn add_interaction(&mut self, question: &str, answer: &str) {
        self.interactions.push(Interaction { 
            question: question.to_string(), 
            answer: answer.to_string(),
         });
    }

    pub fn get_prompt(&self, question: &str) -> String {
        let mut prompt = String::new();
        prompt.push_str("<s>[INST]");
        prompt.push_str(SYSTEM_INSTRUCTION);
        prompt.push_str("[/INST]</s>\n");
        for interaction in &self.interactions {
            prompt.push_str(&interaction.get_prompt());
        }
        prompt.push_str(&format!("<s>[INST]{}[/INST]</s>\n", question));
        prompt
    }
}

impl IntoBody for Conversation {
    fn into_body(self) -> Vec<u8> {
        //todo: room for improvement is exactly here 
        serde_json::to_vec(&self).unwrap_or(vec![])
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Interaction {
    pub question: String,
    pub answer: String,
}

impl Interaction {
    pub fn get_prompt(&self) -> String {
        format!("<s>[INST]{}[/INST]{}</s>\n", self.question, self.answer)
    }
}
