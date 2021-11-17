use crate::format::{format_story, format_story_details};
use api::{ApiClient, PaginationOptions, Story};
use clap::{self, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand};
use state::State;
use std::io::Write;
use std::{
    collections::HashMap,
    error::Error,
    fs::{read_to_string, File},
    io,
};

mod api;
mod format;
mod state;

extern crate reqwest;

const STATE_PATH: &str = ".hn.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .subcommand(
            SubCommand::with_name("top")
                .alias("t")
                .about("Print top stories (default command)"),
        )
        .subcommand(
            SubCommand::with_name("details")
                .alias("d")
                .about("Print a story details")
                .arg(Arg::with_name("INDEX").required(true).help("Story index")),
        )
        .get_matches();

    let mut state = read_state(STATE_PATH);
    match matches.subcommand() {
        ("" | "top", _) => {
            let stories = print_top_stories().await?;
            state.last_stories = Some(stories);
            save_state(&state, STATE_PATH)?;
        }
        ("details", matches) => {
            let last_story = matches
                .and_then(|matches| matches.value_of("INDEX"))
                .and_then(|index_str| result_to_option(index_str.parse::<usize>()))
                .and_then(|index| state.get_last_story(index));
            if let Some(last_story) = last_story {
                print_story_details(last_story.id).await?;
            } else {
                eprintln!("Invalid story index.")
            }
        }
        _ => (),
    };

    Ok(())
}

fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
    result.map(|i| Some(i)).unwrap_or(None)
}

async fn print_top_stories() -> Result<HashMap<usize, Story>, Box<dyn Error>> {
    let api = ApiClient::new();

    let stories_ids = api.top_stories_ids(PaginationOptions::default()).await?;

    let mut stories: HashMap<usize, Story> = HashMap::new();
    for (i, &story_id) in stories_ids.iter().enumerate() {
        let story = api.story_details(story_id).await?;
        println!("{}", format_story(i, &story));
        stories.insert(i + 1, story);
    }
    Ok(stories)
}

async fn print_story_details(id: u32) -> Result<(), Box<dyn Error>> {
    let api = ApiClient::new();

    let story = api.story_details(id).await?;
    println!("{}", format_story_details(&story));
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

fn save_state(state: &State, state_path: &str) -> Result<(), io::Error> {
    let mut file = File::create(state_path)?;
    write!(&mut file, "{}", serde_json::to_string(state).unwrap())?;
    Ok(())
}
