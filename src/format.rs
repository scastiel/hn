use chrono_humanize::HumanTime;
use console::style;
use url::Url;

use crate::api::Story;

pub fn format_story(i: usize, story: &Story) -> String {
    format!(
        "{:2}. ▲ {} {}\n      {}",
        i + 1,
        format_story_title(&story.title),
        story
            .url
            .as_ref()
            .map(|url| format_story_short_url(&url))
            .unwrap_or("".to_string()),
        format_second_line(&story),
    )
}

pub fn format_story_details(story: &Story) -> String {
    format!(
        "▲ {}\n  {}\n  ↳ {}{}",
        format_story_title(&story.title),
        format_second_line(&story),
        story
            .url
            .as_ref()
            .map(|url| format_story_url(&url))
            .unwrap_or("".to_string()),
        story
            .text
            .as_ref()
            .map(|text| format!("\n{}", text))
            .unwrap_or("".to_string()),
    )
}

fn format_story_title(story_title: &str) -> String {
    style(story_title).bold().to_string()
}

fn format_story_short_url(story_url: &Url) -> String {
    style(format!(
        "({})",
        remove_subdomains(story_url.domain().unwrap_or("")).to_string()
    ))
    .dim()
    .to_string()
}

fn format_story_url(story_url: &Url) -> String {
    style(story_url).to_string()
}

fn format_second_line(story: &Story) -> String {
    style(format!(
        "{} points by {} {} | {} comments",
        story.score.unwrap_or(0),
        story.by,
        HumanTime::from(story.time),
        story.descendants.unwrap_or(0)
    ))
    .dim()
    .italic()
    .to_string()
}

fn remove_subdomains(domain: &str) -> &str {
    let mut dot1 = 0;
    let mut dot2 = 0;
    for (i, c) in domain.chars().enumerate() {
        if c == '.' {
            dot1 = dot2;
            dot2 = i;
        }
    }
    dot1 = if dot1 == 0 { 0 } else { dot1 + 1 };
    &domain[dot1..]
}
