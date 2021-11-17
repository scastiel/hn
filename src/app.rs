use crate::api::{ApiClient, Story};
use crate::state::State;
use crate::{api::PaginationOptions, format::format_story};
use std::fs::read_to_string;
use std::io;
use std::{collections::HashMap, error::Error, fs::File, io::Write};

pub struct App {
    pub state_path: String,
    state: State,
}

impl App {
    pub fn new(state_path: &str) -> Self {
        Self {
            state_path: state_path.to_string(),
            state: Self::read_state(state_path),
        }
    }

    async fn print_top_stories(&mut self) -> Result<(), Box<dyn Error>> {
        let api = ApiClient::new();

        let stories_ids = api.top_stories_ids(PaginationOptions::default()).await?;

        let mut stories: HashMap<usize, Story> = HashMap::new();
        for (i, &story_id) in stories_ids.iter().enumerate() {
            let story = api.story_details(story_id).await?;
            println!("{}", format_story(i, &story));
            stories.insert(i, story);
        }
        self.state.last_stories = Some(stories);

        Ok(())
    }

    fn read_state(state_path: &str) -> State {
        if let Ok(state_str) = read_to_string(state_path) {
            if let Ok(state) = serde_json::from_str(state_str.as_str()) {
                return state;
            }
            eprintln!(
                "Warning: unable to deserialize content from {}. Starting from a clean state.",
                state_path
            );
        }
        State::default()
    }

    fn save_state(&self) -> Result<(), io::Error> {
        let mut file = File::create(&self.state_path)?;
        write!(&mut file, "{}", serde_json::to_string(&self.state).unwrap())?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_top_stories().await?;
        self.save_state()?;
        Ok(())
    }
}
