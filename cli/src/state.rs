use std::collections::HashMap;

use chrono::{DateTime, Utc};
use hnapi::Story;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub username: String,
    pub token: String,
    pub expires: DateTime<Utc>,
}

impl Auth {
    pub fn new(username: &str, token: &str, expires: &DateTime<Utc>) -> Auth {
        Auth {
            username: username.to_string(),
            token: token.to_string(),
            expires: *expires,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub last_stories: Option<HashMap<usize, Story>>,
    pub auth: Option<Auth>,
}

impl State {
    pub fn get_last_story(&'_ self, index: usize) -> Option<&'_ Story> {
        if let Some(last_stories) = &self.last_stories {
            last_stories.get(&index)
        } else {
            None
        }
    }
}
