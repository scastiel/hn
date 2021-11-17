use crate::{api::PaginationOptions, format::format_story};
use api::ApiClient;
use std::error::Error;

mod api;
mod format;

extern crate reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api = ApiClient::new();

    let stories_ids = api.top_stories_ids(PaginationOptions::default()).await?;

    for (i, &story_id) in stories_ids.iter().enumerate() {
        let story = api.story_details(story_id).await?;
        println!("{}", format_story(i, &story));
    }

    Ok(())
}
