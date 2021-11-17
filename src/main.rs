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
    let page_arg = Arg::with_name("page")
        .long("page")
        .short("p")
        .takes_value(true)
        .help("Page number");
    let matches = clap::App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .subcommand(
            SubCommand::with_name("top")
                .alias("t")
                .about("Print top stories (default command)")
                .arg(&page_arg),
        )
        .subcommand(
            SubCommand::with_name("new")
                .alias("n")
                .about("Print new stories")
                .arg(&page_arg),
        )
        .subcommand(
            SubCommand::with_name("best")
                .alias("b")
                .about("Print best stories")
                .arg(&page_arg),
        )
        .subcommand(
            SubCommand::with_name("ask")
                .alias("a")
                .about("Print ask stories")
                .arg(&page_arg),
        )
        .subcommand(
            SubCommand::with_name("show")
                .alias("s")
                .about("Print show stories")
                .arg(&page_arg),
        )
        .subcommand(
            SubCommand::with_name("job")
                .alias("j")
                .about("Print best stories")
                .arg(&page_arg),
        )
        .subcommand(
            SubCommand::with_name("details")
                .alias("d")
                .about("Print a story details")
                .arg(Arg::with_name("INDEX").required(true).help("Story index")),
        )
        .subcommand(
            SubCommand::with_name("open")
                .alias("o")
                .about("Open a story’s link in the default browser")
                .arg(Arg::with_name("INDEX").required(true).help("Story index")),
        )
        .get_matches();

    let mut state = read_state(STATE_PATH);
    match matches.subcommand() {
        ("" | "top", matches) => {
            let pagination = get_pagination_from_matches(matches);
            state.last_stories =
                Some(print_stories("topstories", pagination, state.last_stories).await?);
            save_state(&state, STATE_PATH)?;
        }
        ("new", matches) => {
            let pagination = get_pagination_from_matches(matches);
            state.last_stories =
                Some(print_stories("newstories", pagination, state.last_stories).await?);
            save_state(&state, STATE_PATH)?;
        }
        ("best", matches) => {
            let pagination = get_pagination_from_matches(matches);
            state.last_stories =
                Some(print_stories("beststories", pagination, state.last_stories).await?);
            save_state(&state, STATE_PATH)?;
        }
        ("ask", matches) => {
            let pagination = get_pagination_from_matches(matches);
            state.last_stories =
                Some(print_stories("askstories", pagination, state.last_stories).await?);
            save_state(&state, STATE_PATH)?;
        }
        ("show", matches) => {
            let pagination = get_pagination_from_matches(matches);
            state.last_stories =
                Some(print_stories("showstories", pagination, state.last_stories).await?);
            save_state(&state, STATE_PATH)?;
        }
        ("job", matches) => {
            let pagination = get_pagination_from_matches(matches);
            state.last_stories =
                Some(print_stories("jobstories", pagination, state.last_stories).await?);
            save_state(&state, STATE_PATH)?;
        }
        ("details", matches) => {
            let last_story = get_story_from_matches(matches, &state);
            if let Some(last_story) = last_story {
                print_story_details(last_story.id).await?;
            } else {
                eprintln!("Invalid story index.")
            }
        }
        ("open", matches) => {
            let last_story = get_story_from_matches(matches, &state);
            if let Some(last_story) = last_story {
                open_story_link(&last_story).await?;
            } else {
                eprintln!("Invalid story index.")
            }
        }
        _ => (),
    };

    Ok(())
}

fn get_pagination_from_matches(matches: Option<&clap::ArgMatches>) -> PaginationOptions {
    matches
        .and_then(|matches| matches.value_of("page"))
        .and_then(|page_str| result_to_option(page_str.parse::<usize>()))
        .map(|page| PaginationOptions::page(page))
        .unwrap_or(PaginationOptions::default())
}

fn get_story_from_matches<'a>(
    matches: Option<&clap::ArgMatches>,
    state: &'a State,
) -> Option<&'a Story> {
    matches
        .and_then(|matches| matches.value_of("INDEX"))
        .and_then(|index_str| result_to_option(index_str.parse::<usize>()))
        .and_then(|index| state.get_last_story(index))
}

fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
    result.map(|i| Some(i)).unwrap_or(None)
}

async fn print_stories(
    list: &str,
    pagination: PaginationOptions,
    last_stories: Option<HashMap<usize, Story>>,
) -> Result<HashMap<usize, Story>, Box<dyn Error>> {
    let api = ApiClient::new();

    let stories_ids = api.stories_ids(list, &pagination).await?;

    let mut stories = last_stories.unwrap_or(HashMap::new());
    for (i, &story_id) in stories_ids.iter().enumerate() {
        let story = api.story_details(story_id).await?;
        println!("{}", format_story(i + pagination.from, &story));
        stories.insert(i + pagination.from + 1, story);
    }
    Ok(stories)
}

async fn print_story_details(id: u32) -> Result<(), Box<dyn Error>> {
    let api = ApiClient::new();

    let story = api.story_details(id).await?;
    println!("{}", format_story_details(&story));
    Ok(())
}

async fn open_story_link(story: &Story) -> Result<(), Box<dyn Error>> {
    if let Some(url) = &story.url {
        if webbrowser::open(url.as_str()).is_err() {
            eprintln!("Error while opening the default browser.");
        }
    } else {
        eprintln!("No URL is associated to the story.")
    }
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
