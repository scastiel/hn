use crate::tree::SubTree;
use chrono::{DateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, str::FromStr};
use tree::convert;
use url::Url;

extern crate chrono;
extern crate reqwest;
extern crate scraper;
extern crate serde;

mod tree;

const BASE_URL: &str = "https://news.ycombinator.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct Story {
    pub id: u32,
    pub title: String,
    pub url: Url,
    pub url_displayed: Option<String>,
    pub user: Option<String>,
    pub score: Option<u32>,
    pub date: DateTime<Utc>,
    pub date_displayed: String,
    pub comment_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: u32,
    pub user: String,
    pub date: DateTime<Utc>,
    pub date_displayed: String,
    pub html_content: String,
    pub children: Vec<Comment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoryWithDetails {
    pub story: Story,
    pub html_content: Option<String>,
    pub comments: Vec<Comment>,
}

pub enum StoryList {
    News,
    Newest,
    Ask,
    Show,
    Jobs,
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

pub async fn stories_list(
    list: StoryList,
    page: usize,
) -> Result<HashMap<usize, Story>, Box<dyn Error>> {
    let url = format!("{}?p={}", list.url(), page);
    let document = document_at_url(&url).await?;
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

pub async fn story_details(id: u32) -> Result<StoryWithDetails, Box<dyn Error>> {
    let url = format!("{}/item?id={}", BASE_URL, id);
    let document = document_at_url(&url).await?;

    let tr = single_doc_element(&document, "table.fatitem tr.athing").unwrap();
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

    let mut comments_map: HashMap<u32, Comment> = HashMap::new();
    let mut comments_ids_with_indents: Vec<(usize, u32)> = vec![];
    let comments_selector = Selector::parse(".comment-tree tr.comtr").unwrap();
    let comment_trs = document.select(&comments_selector);
    for comment_tr in comment_trs {
        let ind_selector = Selector::parse(".ind").unwrap();
        let indent = comment_tr
            .select(&ind_selector)
            .nth(0)
            .and_then(|ind| ind.value().attr("indent"))
            .map(|ind| ind.parse::<usize>().unwrap())
            .unwrap_or(0);
        let comment = extract_comment_info(&comment_tr);
        comments_ids_with_indents.push((indent, comment.id));
        comments_map.insert(comment.id, comment);
    }
    let comments = make_comments_tree(&comments_ids_with_indents, &mut comments_map);

    let story_details = StoryWithDetails {
        story,
        html_content,
        comments,
    };
    Ok(story_details)
}

fn make_comments_tree(
    comments_ids_with_indents: &Vec<(usize, u32)>,
    comments_map: &mut HashMap<u32, Comment>,
) -> Vec<Comment> {
    let tree = convert(&comments_ids_with_indents);
    fn get_comment(subtree: &SubTree<u32>, comments: &mut HashMap<u32, Comment>) -> Comment {
        let mut comment = comments.remove(&subtree.val).unwrap();
        comment.children = subtree
            .children
            .iter()
            .map(|subtree| get_comment(subtree, comments))
            .collect();
        comment
    }
    let comments: Vec<Comment> = tree
        .children
        .iter()
        .map(|subtree| get_comment(subtree, comments_map))
        .collect();
    comments
}

async fn document_at_url(url: &str) -> Result<Html, reqwest::Error> {
    let resp = reqwest::get(url).await?;
    let html = resp.text().await?;
    Ok(Html::parse_document(&html))
}

fn extract_story_info(first_line_el: &ElementRef) -> Story {
    let id = first_line_el.value().attr("id").unwrap().parse().unwrap();
    let title_el = single_element(&first_line_el, ".titlelink").unwrap();
    let (title, url) = link_info(&title_el);
    let url_displayed = single_element_html(&first_line_el, ".sitestr");

    let second_line_el = ElementRef::wrap(first_line_el.next_sibling().unwrap()).unwrap();
    let score = single_element_html(&second_line_el, ".score").map(parse_score);
    let user = single_element_html(&second_line_el, ".hnuser");
    let (date, date_displayed) = single_element(&second_line_el, ".age")
        .map(|d| date_info(&d))
        .unwrap();

    let comment_count = second_line_el
        .select(&Selector::parse("a").unwrap())
        .filter(|el| el.inner_html().contains("&nbsp;comment"))
        .nth(0)
        .map(|el| parse_comment_count(el.inner_html()));

    Story {
        id,
        title,
        url,
        url_displayed,
        user,
        score,
        date,
        date_displayed,
        comment_count,
    }
}

fn extract_comment_info(comment_el: &ElementRef) -> Comment {
    let id = comment_el.value().attr("id").unwrap().parse().unwrap();

    let user = single_element_html(&comment_el, ".hnuser").unwrap();
    let (date, date_displayed) = single_element(&comment_el, ".age")
        .map(|d| date_info(&d))
        .unwrap();

    let html_content = single_element(&comment_el, ".commtext")
        .map(|el| {
            let first_paragraph = el.text().nth(0).unwrap_or("");
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
        children: vec![],
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
    document.select(&Selector::parse(selector).unwrap()).nth(0)
}

fn single_element<'a>(el: &'a ElementRef, selector: &str) -> Option<ElementRef<'a>> {
    el.select(&Selector::parse(selector).unwrap()).nth(0)
}

fn single_element_html(el: &ElementRef, selector: &str) -> Option<String> {
    single_element(&el, selector).map(|el| el.inner_html())
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
    use std::error::Error;

    #[tokio::test]
    async fn top_stories_return_something() -> Result<(), Box<dyn Error>> {
        let res = stories_list(StoryList::News, 1).await?;
        assert_eq!(res.len(), 30);
        Ok(())
    }

    #[tokio::test]
    async fn story_details_return_something() -> Result<(), Box<dyn Error>> {
        let details = story_details(27883047).await?;
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

        assert!(details.comments.len() > 0);

        println!("{:#?}", details);

        Ok(())
    }

    #[tokio::test]
    async fn story_details_return_something_with_text() -> Result<(), Box<dyn Error>> {
        let details = story_details(29246573).await?;
        assert_eq!(details.html_content, Some("I have been programming web and backend stuff for over a decade but I have never done any kind of image processing. I tried googling but there is so much noise in the QR space.<p>What I want to know is, how does QR scanner code work? How do you go from a photo of a QR to the encoded text within, allowing for all of the factors that will get in the way like poor quality cameras, off-center photos, blurriness etc? Is there a code-first tutorial or worked example somewhere?</p>".to_string()));
        Ok(())
    }
}
