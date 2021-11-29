use std::collections::HashMap;

use hnapi::Story;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Auth {
    pub username: String,
    pub token: String,
}

impl Auth {
    pub fn new(username: &str, token: &str) -> Auth {
        Auth {
            username: username.to_string(),
            token: token.to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub last_stories: Option<HashMap<usize, Story>>,
    pub auth: Option<Auth>,
}

impl State {
    pub fn get_last_story<'a>(&'a self, index: usize) -> Option<&'a Story> {
        if let Some(last_stories) = &self.last_stories {
            last_stories.get(&index)
        } else {
            None
        }
    }
}
