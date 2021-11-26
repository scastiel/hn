use console::style;
use hnapi::{Comment, Story, StoryWithDetails, User};
use html_escape::decode_html_entities;
use hyphenation::{Language, Load, Standard};
use regex::Regex;
use textwrap::{fill, Options};
use url::Url;

pub fn format_user(user: &User) -> String {
    format!(
        "{}{}\n{}{}\n{}{}\n{}{}\n",
        style("user:    ").dim(),
        user.id,
        style("created: ").dim(),
        user.created.format("%v").to_string().trim(),
        style("karma:   ").dim(),
        user.karma,
        style("about:   ").dim(),
        format_story_text(&user.about, 0)
            .lines()
            .enumerate()
            .map(|(i, line)| if i == 0 {
                line.to_string()
            } else {
                format!("         {}", line)
            })
            .collect::<Vec<String>>()
            .join("\n"),
    )
}

pub fn format_story(rank: usize, story: &Story) -> String {
    format!(
        "{:2}. ▲ {} {}\n      {}",
        rank,
        format_story_title(&story.title),
        format_story_short_url(&story),
        format_second_line(&story),
    )
}

pub fn format_story_details(details: &StoryWithDetails) -> String {
    format!(
        "▲ {}\n  {}{}{}",
        format_story_title(&details.story.title),
        format_second_line(&details.story),
        format!("\n  ↳ {}", format_story_url(&details.story.url)),
        details
            .html_content
            .as_deref()
            .map(|text| format!("\n\n{}", format_story_text(&text, 0)))
            .unwrap_or("".to_string()),
    )
}

pub fn format_comment(comment: &Comment, level: usize) -> String {
    format!(
        "{}\n{}",
        indent(&format_comment_header(&comment), level),
        format_story_text(&comment.html_content, level),
    )
}

pub fn indent(text: &str, level: usize) -> String {
    text.lines()
        .map(|line| format!("{}{}", "  ".repeat(level), line))
        .collect::<Vec<String>>()
        .join("\n")
}

fn format_story_text(text: &str, level: usize) -> String {
    let text = text.replace("<p>", "\n\n").replace("</p>", "");
    let text = decode_html_entities(&text);
    let text = Regex::new("<a [^>]*href=\".*\">(.*)</a>")
        .unwrap()
        .replace_all(&text, style("$1").dim().to_string());
    let text = Regex::new("<i>(.*)</i>")
        .unwrap()
        .replace_all(&text, style("$1").italic().to_string());
    indent(&wrap_text(&text.trim(), 80 - level * 2), level)
}

fn wrap_text(text: &str, width: usize) -> String {
    let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
    let options = Options::new(width).word_splitter(dictionary);
    fill(&text, &options)
}

fn format_story_title(story_title: &str) -> String {
    style(story_title).bold().to_string()
}

fn format_story_short_url(story: &Story) -> String {
    story
        .url_displayed
        .as_deref()
        .map(|url| style(format!("({})", url)).dim().to_string())
        .unwrap_or_default()
}

fn format_story_url(story_url: &Url) -> String {
    style(story_url).to_string()
}

fn format_second_line(story: &Story) -> String {
    style(format!(
        "{} points{} {} | {} comments",
        story.score.unwrap_or(0),
        story
            .user
            .as_deref()
            .map(|by| format!(" by {}", by))
            .unwrap_or("".to_string()),
        story.date_displayed,
        story.comment_count.unwrap_or(0)
    ))
    .dim()
    .italic()
    .to_string()
}

fn format_comment_header(comment: &Comment) -> String {
    style(format!("{} {}", comment.user, comment.date_displayed,))
        .dim()
        .italic()
        .to_string()
}
