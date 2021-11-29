use crate::format::{format_comment, format_story, format_story_details, format_user};
use crate::state::Auth;
use clap::{self, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand};
use console::style;
use hnapi::{login, stories_list, story_details, user_details, Comment, Story, StoryList};
use minus::Pager;
use state::State;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::{
    collections::HashMap,
    error::Error,
    fs::{read_to_string, File},
};

mod format;
mod state;

extern crate reqwest;

fn get_state_path() -> String {
    dirs::home_dir()
        .and_then(|home_dir| home_dir.to_str().map(ToString::to_string))
        .map(|home_dir| format!("{}/.hn.json", home_dir))
        .expect("Can’t get home directory.")
}

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
        .subcommand(
            SubCommand::with_name("user")
                .alias("u")
                .about("Show details about a user")
                .arg(Arg::with_name("USER_NAME").required(true).help("User name")),
        )
        .subcommand(SubCommand::with_name("login").alias("l"))
        .subcommand(SubCommand::with_name("logout"))
        .get_matches();

    let state_path = get_state_path();
    let mut state = read_state(&state_path);
    match matches.subcommand() {
        ("" | "top", matches) => {
            let page = get_page_from_matches(matches);
            state.last_stories =
                Some(print_stories(StoryList::News, page, state.last_stories).await?);
            save_state(&state, &state_path)?;
        }
        ("new", matches) => {
            let page = get_page_from_matches(matches);
            state.last_stories =
                Some(print_stories(StoryList::Newest, page, state.last_stories).await?);
            save_state(&state, &state_path)?;
        }
        ("best", matches) => {
            let page = get_page_from_matches(matches);
            state.last_stories =
                Some(print_stories(StoryList::Best, page, state.last_stories).await?);
            save_state(&state, &state_path)?;
        }
        ("ask", matches) => {
            let page = get_page_from_matches(matches);
            state.last_stories =
                Some(print_stories(StoryList::Ask, page, state.last_stories).await?);
            save_state(&state, &state_path)?;
        }
        ("show", matches) => {
            let page = get_page_from_matches(matches);
            state.last_stories =
                Some(print_stories(StoryList::Show, page, state.last_stories).await?);
            save_state(&state, &state_path)?;
        }
        ("job", matches) => {
            let page = get_page_from_matches(matches);
            state.last_stories =
                Some(print_stories(StoryList::Jobs, page, state.last_stories).await?);
            save_state(&state, &state_path)?;
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
        ("user", matches) => {
            let user_id = matches
                .and_then(|matches| matches.value_of("USER_NAME"))
                .unwrap();
            if let Some(user) = user_details(user_id).await? {
                println!("{}", format_user(&user));
            } else {
                eprintln!("Invalid user name.")
            }
        }
        ("login", _) => {
            if let Some(auth) = state.auth {
                println!("Already signed in as {}.", style(&auth.username).bold());
            } else {
                let username = prompt("Username: ")?;
                let password = prompt("Password: ")?;
                let token = login(&username, &password).await?;
                if let Some(token) = token {
                    println!("Successfully signed in as {}.", style(&username).bold());
                    state.auth = Some(Auth::new(&username, &token));
                    save_state(&state, &state_path)?;
                } else {
                    println!("Invalid username or password.");
                }
            }
        }
        ("logout", _) => {
            if state.auth.is_some() {
                state.auth = None;
                save_state(&state, &state_path)?;
                println!("Signed out.");
            } else {
                println!("Not signed in.");
            }
        }
        _ => (),
    };

    Ok(())
}

fn prompt(prompt: &str) -> Result<String, std::io::Error> {
    let stdin = std::io::stdin();
    let mut input = String::new();
    print!("{}", prompt);
    std::io::stdout().flush()?;
    stdin
        .read_line(&mut input)
        .expect("Can’t read standard input.");
    Ok(input.trim_end().to_string())
}

fn get_page_from_matches(matches: Option<&clap::ArgMatches>) -> usize {
    matches
        .and_then(|matches| matches.value_of("page"))
        .and_then(|page_str| result_to_option(page_str.parse::<usize>()))
        .unwrap_or(1)
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
    list: StoryList,
    page: usize,
    last_stories: Option<HashMap<usize, Story>>,
) -> Result<HashMap<usize, Story>, Box<dyn Error>> {
    let stories = stories_list(list, page).await?;
    let mut last_stories = last_stories.unwrap_or(HashMap::new());
    let mut ranks: Vec<usize> = stories.keys().copied().collect();
    ranks.sort();
    for rank in ranks {
        let story = stories.get(&rank).unwrap();
        println!("{}", format_story(rank, &story));
    }
    last_stories.extend(stories);
    Ok(last_stories)
}

async fn print_story_details(id: u32) -> Result<(), Box<dyn Error>> {
    let mut output = Pager::new().unwrap();
    output.set_prompt("More");

    let details = story_details(id).await?.unwrap();
    writeln!(output, "{}", format_story_details(&details))?;

    let comments = details.comments;
    for comment in comments {
        print_comment(&mut output, &comment, 0)?;
    }

    minus::page_all(output)?;

    Ok(())
}

fn print_comment<'a>(
    output: &'a mut Pager,
    comment: &'a Comment,
    level: usize,
) -> Result<(), Box<dyn Error>> {
    writeln!(output, "\n{}", format_comment(&comment, level))?;
    let children = comment.children.borrow();
    for child_comment in children.iter() {
        print_comment(output, child_comment, level + 1)?;
    }

    Ok(())
}

async fn open_story_link(story: &Story) -> Result<(), Box<dyn Error>> {
    if webbrowser::open(story.url.as_str()).is_err() {
        eprintln!("Error while opening the default browser.");
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

fn save_state(state: &State, state_path: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(state_path)?;
    write!(&mut file, "{}", serde_json::to_string(state).unwrap())?;
    Ok(())
}
