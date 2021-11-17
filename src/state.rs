use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::Story;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub last_stories: Option<HashMap<usize, Story>>,
}
