use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use url::Url;

const DEFAULT_NUMBER_OF_ITEMS_PER_PAGE: usize = 10;

#[derive(Clone)]
pub struct PaginationOptions {
    pub from: usize,
    pub to: usize,
}

impl PaginationOptions {
    pub fn default() -> Self {
        Self {
            from: 0,
            to: DEFAULT_NUMBER_OF_ITEMS_PER_PAGE,
        }
    }

    pub fn from(&self, from: usize) -> Self {
        Self { from, ..*self }
    }

    pub fn to(&self, to: usize) -> Self {
        Self { to, ..*self }
    }

    pub fn page(page: usize) -> Self {
        Self {
            from: (page - 1) * DEFAULT_NUMBER_OF_ITEMS_PER_PAGE,
            to: page * DEFAULT_NUMBER_OF_ITEMS_PER_PAGE,
        }
    }
}

pub struct ApiClient {
    client: reqwest::Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    #[serde(rename = "job")]
    Job,
    #[serde(rename = "story")]
    Story,
    #[serde(rename = "comment")]
    Comment,
    #[serde(rename = "poll")]
    Poll,
    #[serde(rename = "pollopt")]
    PollOpt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Story {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(rename = "type")]
    pub type_: Type,
    pub by: String,
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
    pub text: Option<String>,
    #[serde(default)]
    pub dead: bool,
    pub parent: Option<u32>,
    pub poll: Option<u32>,
    pub kids: Option<Vec<u32>>,
    pub url: Option<Url>,
    pub score: Option<u32>,
    pub title: String,
    pub parts: Option<u32>,
    pub descendants: Option<u32>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn stories_ids(
        &self,
        list: &str,
        pagination: &PaginationOptions,
    ) -> Result<Vec<u32>, reqwest::Error> {
        let url = format!("https://hacker-news.firebaseio.com/v0/{}.json", list);
        let ids = self.json::<Vec<u32>>(url.as_str()).await?;
        let ids = ids[pagination.from..pagination.to].to_vec();
        Ok(ids)
    }

    pub async fn story_details(&self, id: u32) -> Result<Story, reqwest::Error> {
        let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
        Ok(self.json::<Story>(&url).await?)
    }

    async fn json<T: DeserializeOwned>(&self, url: &str) -> Result<T, reqwest::Error> {
        Ok(self.client.get(url).send().await?.json::<T>().await?)
    }
}
