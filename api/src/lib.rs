//! Use this crate to query stories from [HackerNews](https://news.ycombinator.com/).
//!
//! For now, it supports the following operations:
//!   - list stories using [`stories_list`]
//!   - get details and comments for a story using [`story_details`]
//!   - get details about a user using [`user_details`]
//!   - login and get an auth token using [`login`]
//!   - upvote a story using [`upvote_story`]
//!
//! Refer to their respective documentations to see usage examples.
//!
//! **Note:** information is obtained by scraping the HackerNews website. The reason this crate
//! does not use the [official API](https://github.com/HackerNews/API) is that it does
//! not provide a convenient way to get all the comments for a given story, and only allows
//! read operations.

use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use reqwest::header::COOKIE;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    rc::{Rc, Weak},
    str::FromStr,
};
use url::Url;

extern crate chrono;
extern crate reqwest;
extern crate scraper;
extern crate serde;
extern crate url;

const BASE_URL: &str = "https://news.ycombinator.com";

#[derive(Debug, Serialize, Deserialize)]
/// Information about a story.
pub struct Story {
    /// ID of the story.
    pub id: u32,
    /// Story title.
    pub title: String,
    /// Story full URL. For the text stories, the URL will be on “news.ycombinator.org”.
    pub url: Url,
    /// URL as it is display. Often the domain only (e.g. “google.com”), possibly with
    /// additions (e.g. “github.com/scastiel”).
    pub url_displayed: Option<String>,
    /// Parameter to give to `upvote` method to be able to upvote a story. Will be None if
    /// not logged in.
    pub upvote_auth: Option<String>,
    /// Nickname of the user who posted the story.
    pub user: Option<String>,
    /// Score of the story at this instant.
    pub score: Option<u32>,
    /// Date the story was posted.
    pub date: DateTime<Utc>,
    /// Date the story was posted, as it is displayed, e.g. “2 months ago”.
    pub date_displayed: String,
    /// Number of comments posted on the story.
    pub comment_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Information about a user.
pub struct User {
    // User ID (their username).
    pub id: String,
    // Creation date.
    pub created: NaiveDate,
    // Karma.
    pub karma: u32,
    // About text (biography).
    pub about: String,
}

#[derive(Debug)]
/// Comment posted on a story. A comment can have a parent if it is a reply
/// to another comment, and can have children.
pub struct Comment {
    /// ID of the comment.
    pub id: u32,
    /// User who posted the comment.
    pub user: String,
    /// Date the comment was posted.
    pub date: DateTime<Utc>,
    /// Date the comment was posted, as it is displayed, e.g. “2 months ago”.
    pub date_displayed: String,
    /// HTML content of the comment.
    pub html_content: String,
    /// Parent comment, if any.
    pub parent: RefCell<Option<Weak<Comment>>>,
    /// Reply comments.
    pub children: RefCell<Vec<Rc<Comment>>>,
}

#[derive(Debug)]
/// Combination of a story, its HTML content, and its comments.
pub struct StoryWithDetails {
    /// Information about the story.
    pub story: Story,
    /// HTML content of the story.
    pub html_content: Option<String>,
    /// List of the comments posted on the story.
    pub comments: Vec<Rc<Comment>>,
}

/// Available story lists.
pub enum StoryList {
    /// Top stories.
    News,
    /// New stories.
    Newest,
    /// “Ask HN” stories.
    Ask,
    /// “Show HN” stories.
    Show,
    /// Job stories.
    Jobs,
    /// Best stories.
    Best,
}

impl StoryList {
    pub fn url(&self) -> String {
        match self {
            StoryList::News => format!("{}/news", BASE_URL),
            StoryList::Newest => format!("{}/newest", BASE_URL),
            StoryList::Ask => format!("{}/ask", BASE_URL),
            StoryList::Show => format!("{}/show", BASE_URL),
            StoryList::Jobs => format!("{}/jobs", BASE_URL),
            StoryList::Best => format!("{}/best", BASE_URL),
        }
    }
}

/// Get all the stories for a given list at a given page.
///
/// ## Example
///
/// ```
/// use hnapi::{stories_list, StoryList};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let stories = stories_list(StoryList::News, 1, &None).await?;
///     assert_eq!(stories.len(), 30);
///     let first_story = stories.get(&1);
///     assert!(first_story.is_some());
///     let first_story = first_story.unwrap();
///     println!("{:#?}", first_story);
///     Ok(())
/// }
/// ```
pub async fn stories_list(
    list: StoryList,
    page: usize,
    token: &Option<String>,
) -> Result<HashMap<usize, Story>, Box<dyn Error>> {
    let url = format!("{}?p={}", list.url(), page);
    let document = document_at_url(&url, token).await?;
    let stories: HashMap<usize, Story> = document
        .select(&Selector::parse("tr.athing").unwrap())
        .map(|tr| {
            let rank = single_element_html(&tr, ".rank")
                .map(|rank| rank.replace(".", "").parse::<usize>().unwrap())
                .unwrap();
            let story = extract_story_info(&tr);
            (rank, story)
        })
        .collect();
    Ok(stories)
}

/// Get the details about a given story. Will return `null` for a non-existent story ID.
///
/// ## Example
///
/// ```
/// use hnapi::story_details;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let details = story_details(29203502).await?;
///     assert!(details.is_some());
///     let details = details.unwrap();
///     println!("{:#?}", details);
///     assert!(details.story.title.contains("Lifeee"));
///     assert_eq!(details.story.user, Some("scastiel".to_string()));
///     assert!(details.html_content.is_none());
///     assert!(details.comments.len() > 10);
///     Ok(())
/// }
/// ```
pub async fn story_details(id: u32) -> Result<Option<StoryWithDetails>, Box<dyn Error>> {
    let url = format!("{}/item?id={}", BASE_URL, id);
    let document = document_at_url(&url, &None).await?;
    if let Some(tr) = single_doc_element(&document, "table.fatitem tr.athing") {
        let story = extract_story_info(&tr);

        let html_content = tr
            .next_sibling()
            .and_then(|el| el.next_sibling())
            .and_then(|el| el.next_sibling())
            .and_then(|el| el.next_sibling())
            .and_then(|el| el.first_child())
            .and_then(|el| el.next_sibling())
            .and_then(ElementRef::wrap)
            .map(|el| el.inner_html())
            .filter(|html| !html.contains("<form "));

        // let mut comments_map: HashMap<u32, Comment> = HashMap::new();
        // let mut comments_ids_with_indents: Vec<(usize, u32)> = vec![];
        let comments_selector = Selector::parse(".comment-tree tr.comtr").unwrap();
        let comment_trs = document.select(&comments_selector);
        let mut comments: Vec<Rc<Comment>> = vec![];
        let mut parent_stack: Vec<Rc<Comment>> = vec![];
        for comment_tr in comment_trs {
            let ind_selector = Selector::parse(".ind").unwrap();
            let indent = comment_tr
                .select(&ind_selector)
                .next()
                .and_then(|ind| ind.value().attr("indent"))
                .map(|ind| ind.parse::<usize>().unwrap())
                .unwrap_or(0);
            let comment = Rc::new(extract_comment_info(&comment_tr));

            while indent < parent_stack.len() {
                parent_stack.pop();
            }

            if indent == 0 {
                comments.push(Rc::clone(&comment));
                parent_stack.push(Rc::clone(&comment));
            } else {
                let parent = parent_stack.pop().unwrap();
                (*parent.children.borrow_mut()).push(Rc::clone(&comment));
                (*comment.parent.borrow_mut()) = Some(Rc::downgrade(&parent));
                parent_stack.push(parent);
                parent_stack.push(comment);
            }
        }

        let story_details = StoryWithDetails {
            story,
            html_content,
            comments,
        };
        Ok(Some(story_details))
    } else {
        Ok(None)
    }
}

/// Get the details about a given user. Will return `null` for a non-existent user ID.
///
/// ## Example
///
/// ```
/// use hnapi::user_details;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let user = user_details("scastiel").await?;
///     assert!(user.is_some());
///     let user = user.unwrap();
///     println!("{:#?}", user);
///     assert_eq!(user.id, "scastiel".to_string());
///     Ok(())
/// }
/// ```
pub async fn user_details(id: &str) -> Result<Option<User>, Box<dyn Error>> {
    let url = format!("{}/user?id={}", BASE_URL, id);
    let document = document_at_url(&url, &None).await?;
    if let Some(table) =
        single_doc_element(&document, "#hnmain > tbody > tr:nth-child(3) > td > table")
    {
        let id = single_element_html(&table, "tr:nth-child(1) .hnuser").unwrap();

        let created = single_element(&table, "tr:nth-child(2) > td:nth-child(2) > a")
            .and_then(|a| a.value().attr("href"))
            .map(|href| {
                let caps = Regex::new(r"(?P<date>\d{4}-\d{2}-\d{2})")
                    .unwrap()
                    .captures(href)
                    .unwrap();
                NaiveDate::from_str(&caps["date"]).unwrap()
            })
            .unwrap();

        let karma = single_element_html(&table, "tr:nth-child(3) > td:nth-child(2)")
            .map(|karma| karma.trim().parse().unwrap())
            .unwrap();
        let about = single_element_html(&table, "tr:nth-child(4) > td:nth-child(2)")
            .map(|about| about.trim().to_string())
            .unwrap();

        return Ok(Some(User {
            id,
            created,
            karma,
            about,
        }));
    }
    Ok(None)
}

async fn document_at_url(url: &str, token: &Option<String>) -> Result<Html, reqwest::Error> {
    let client = reqwest::ClientBuilder::new().build()?;
    let mut request_builder = client.get(url);
    if let Some(token) = token {
        request_builder = request_builder.header(COOKIE, format!("user={}", token));
    }
    let resp = request_builder.send().await?;
    let html = resp.text().await?;
    Ok(Html::parse_document(&html))
}

pub async fn login(
    username: &str,
    password: &str,
) -> Result<Option<(String, DateTime<Utc>)>, reqwest::Error> {
    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let url = format!("{}/login", BASE_URL);
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("goto", "news")
        .append_pair("acct", username)
        .append_pair("pw", password)
        .finish();
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;
    let token = response.cookies().next().map(|cookie| {
        let token = cookie.value().to_string();
        let expires = cookie.expires().map(DateTime::<Utc>::from).unwrap();
        (token, expires)
    });
    Ok(token)
}

pub async fn upvote_story(id: u32, upvote_auth: &str, token: &str) -> Result<bool, reqwest::Error> {
    let url = format!(
        "{}/vote?id={}&how=up&auth={}&goto=news",
        BASE_URL, id, upvote_auth
    );
    let document = document_at_url(&url, &Some(token.to_string())).await?;
    if single_doc_element(&document, "form[action='vote']").is_some() {
        return Ok(false);
    }
    Ok(true)
}

fn extract_story_info(first_line_el: &ElementRef) -> Story {
    let id = first_line_el.value().attr("id").unwrap().parse().unwrap();
    let title_el = single_element(first_line_el, ".titlelink").unwrap();
    let (title, url) = link_info(&title_el);
    let url_displayed = single_element_html(first_line_el, ".sitestr");
    let upvote_auth = single_element(first_line_el, ".clicky").and_then(|upvote_link| {
        let (_, upvote_url) = link_info(&upvote_link);
        upvote_url.query_pairs().find_map(|(key, value)| {
            if key == "auth" {
                Some(value.to_string())
            } else {
                None
            }
        })
    });

    let second_line_el = ElementRef::wrap(first_line_el.next_sibling().unwrap()).unwrap();
    let score = single_element_html(&second_line_el, ".score").map(parse_score);
    let user = single_element_html(&second_line_el, ".hnuser");
    let (date, date_displayed) = single_element(&second_line_el, ".age")
        .map(|d| date_info(&d))
        .unwrap();

    let comment_count = second_line_el
        .select(&Selector::parse("a").unwrap())
        .find(|el| el.inner_html().contains("&nbsp;comment"))
        .map(|el| parse_comment_count(el.inner_html()));

    Story {
        id,
        title,
        url,
        url_displayed,
        upvote_auth,
        user,
        score,
        date,
        date_displayed,
        comment_count,
    }
}

fn extract_comment_info(comment_el: &ElementRef) -> Comment {
    let id = comment_el.value().attr("id").unwrap().parse().unwrap();

    let user = single_element_html(comment_el, ".hnuser").unwrap();
    let (date, date_displayed) = single_element(comment_el, ".age")
        .map(|d| date_info(&d))
        .unwrap();

    let html_content = single_element(comment_el, ".commtext")
        .map(|el| {
            let first_paragraph = el.text().next().unwrap_or("");
            let other_paragraphes = el
                .children()
                .flat_map(ElementRef::wrap)
                .filter(|el| el.value().attr("class") != Some("reply"))
                .map(|el| el.html())
                .collect::<Vec<_>>()
                .join("");
            format!("{}{}", first_paragraph, other_paragraphes)
        })
        .unwrap();

    Comment {
        id,
        user,
        date,
        date_displayed,
        html_content,
        parent: RefCell::new(None),
        children: RefCell::new(vec![]),
    }
}

fn parse_score(score: String) -> u32 {
    score
        .replace(" points", "")
        .replace(" point", "")
        .parse()
        .unwrap()
}

fn parse_comment_count(comment_count: String) -> u32 {
    comment_count
        .replace("&nbsp;comments", "")
        .replace("&nbsp;comment", "")
        .parse()
        .unwrap()
}

fn single_doc_element<'a>(document: &'a Html, selector: &str) -> Option<ElementRef<'a>> {
    document.select(&Selector::parse(selector).unwrap()).next()
}

fn single_element<'a>(el: &'a ElementRef, selector: &str) -> Option<ElementRef<'a>> {
    el.select(&Selector::parse(selector).unwrap()).next()
}

fn single_element_html(el: &ElementRef, selector: &str) -> Option<String> {
    single_element(el, selector).map(|el| el.inner_html())
}

fn link_info(link_el: &ElementRef) -> (String, Url) {
    let inner_html = link_el.inner_html();
    let link = link_el.value().attr("href").unwrap();
    let url = if let Ok(url) = Url::from_str(link) {
        url
    } else {
        Url::from_str(format!("{}/{}", BASE_URL, link).as_str()).unwrap()
    };
    (inner_html, url)
}

fn date_info(date_el: &ElementRef) -> (DateTime<Utc>, String) {
    let date =
        DateTime::from_str(&format!("{}.000Z", date_el.value().attr("title").unwrap())).unwrap();
    let date_displayed = ElementRef::wrap(date_el.first_child().unwrap())
        .unwrap()
        .inner_html();
    (date, date_displayed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::error::Error;

    #[tokio::test]
    #[serial]
    async fn top_stories_return_something() -> Result<(), Box<dyn Error>> {
        let res = stories_list(StoryList::News, 1, &None).await?;
        assert_eq!(res.len(), 30);
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn story_details_return_something() -> Result<(), Box<dyn Error>> {
        let details = story_details(27883047).await?.unwrap();

        let comment = details.comments.get(1).unwrap();
        let children = comment.children.borrow();
        let child = (*children).get(0).unwrap();
        let child_parent = child.parent.borrow().as_ref().unwrap().upgrade().unwrap();
        assert_eq!(child_parent.id, comment.id);

        assert_eq!(details.story.id, 27883047);
        assert_eq!(
            details.story.title,
            "Julia Computing raises $24M Series A".to_string()
        );
        assert_eq!(details.story.url.to_string(), "https://www.hpcwire.com/off-the-wire/julia-computing-raises-24m-series-a-former-snowflake-ceo-bob-muglia-joins-board/".to_string());
        assert_eq!(details.story.url_displayed, Some("hpcwire.com".to_string()));
        assert!(details.story.score.is_some() && details.story.score.unwrap() > 0);
        assert_eq!(details.story.user, Some("dklend122".to_string()));
        assert_eq!(
            details.story.date,
            DateTime::<Utc>::from_str("2021-07-19T14:33:05.000Z").unwrap()
        );
        assert!(details.story.date_displayed.contains("ago"));
        assert!(details.story.comment_count.is_some() && details.story.comment_count.unwrap() > 0);

        assert_eq!(details.html_content, None);

        assert!(!details.comments.is_empty());

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn story_details_return_something_with_text() -> Result<(), Box<dyn Error>> {
        let details = story_details(29246573).await?.unwrap();
        assert_eq!(details.html_content, Some("I have been programming web and backend stuff for over a decade but I have never done any kind of image processing. I tried googling but there is so much noise in the QR space.<p>What I want to know is, how does QR scanner code work? How do you go from a photo of a QR to the encoded text within, allowing for all of the factors that will get in the way like poor quality cameras, off-center photos, blurriness etc? Is there a code-first tutorial or worked example somewhere?</p>".to_string()));
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn user_details_returns_none_for_nonexistent_id() -> Result<(), Box<dyn Error>> {
        let user = user_details("ihopethisusernamedoesnotexist").await?;
        assert!(user.is_none());
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn user_details_returns_details_for_existent_id() -> Result<(), Box<dyn Error>> {
        let user = user_details("scastiel").await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, "scastiel".to_string());
        assert_eq!(user.created, NaiveDate::from_ymd(2019, 2, 16));
        assert!(user.karma > 0);
        assert!(user.about.len() > 0);
        Ok(())
    }
}
