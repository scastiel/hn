use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::Story;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub last_stories: Option<HashMap<usize, Story>>,
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
