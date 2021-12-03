#[macro_use]
extern crate juniper;

use std::rc::Rc;

use juniper::{EmptySubscription, FieldError, GraphQLObject, RootNode};
use warp::{hyper::Uri, Filter};

#[derive(GraphQLObject)]
/// Information about a story.
struct Story {
    /// ID of the story.
    pub id: i32,
    /// Story title.
    pub title: String,
    /// Story full URL. For the text stories, the URL will be on “news.ycombinator.org”.
    pub url: String,
    /// URL as it is display. Often the domain only (e.g. “google.com”), possibly with
    /// additions (e.g. “github.com/scastiel”).
    pub url_displayed: Option<String>,
    /// Parameter to give to `upvote` mutation to be able to upvote a story. Will be null if
    /// not logged in.
    pub upvote_auth: Option<String>,
    /// Nickname of the user who posted the story.
    pub user: Option<String>,
    /// Score of the story at this instant.
    pub score: Option<i32>,
    /// Date the story was posted.
    pub date: String,
    /// Date the story was posted, as it is displayed, e.g. “2 months ago”.
    pub date_displayed: String,
    /// Number of comments posted on the story.
    pub comment_count: Option<i32>,
}

impl Story {
    pub fn from_api_story(story: &hnapi::Story) -> Story {
        Story {
            id: story.id as i32,
            title: story.title.clone(),
            url: story.url.to_string(),
            url_displayed: story.url_displayed.clone(),
            upvote_auth: story.upvote_auth.clone(),
            user: story.user.clone(),
            score: story.score.map(|score| score as i32),
            date: story.date.to_string(),
            date_displayed: story.date_displayed.clone(),
            comment_count: story.comment_count.map(|score| score as i32),
        }
    }
}

#[derive(GraphQLObject)]
/// Combination of a story and the rank at which it is displayed, depending on
/// the request returning the story.
struct StoryWithRank {
    /// Story rank.
    pub rank: i32,
    /// Information about the story.
    pub story: Story,
}

impl StoryWithRank {
    pub fn from_api_story(rank: usize, story: &hnapi::Story) -> StoryWithRank {
        StoryWithRank {
            rank: rank as i32,
            story: Story::from_api_story(story),
        }
    }
}

#[derive(GraphQLEnum)]
/// Available story lists.
enum StoryList {
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

impl Default for StoryList {
    fn default() -> StoryList {
        StoryList::News
    }
}

impl StoryList {
    pub fn to_api_story_list(&self) -> hnapi::StoryList {
        match self {
            StoryList::News => hnapi::StoryList::News,
            StoryList::Newest => hnapi::StoryList::Newest,
            StoryList::Ask => hnapi::StoryList::Ask,
            StoryList::Show => hnapi::StoryList::Show,
            StoryList::Jobs => hnapi::StoryList::Jobs,
            StoryList::Best => hnapi::StoryList::Best,
        }
    }
}

#[derive(GraphQLObject)]
/// Comment posted on a story. A comment can have a parent if it is a reply
/// to another comment, and can have children.
struct Comment {
    /// ID of the parent comment (null if posted on the story).
    pub parent: Option<i32>,
    /// ID of the comment.
    pub id: i32,
    /// User who posted the comment.
    pub user: String,
    /// Date the comment was posted.
    pub date: String,
    /// Date the comment was posted, as it is displayed, e.g. “2 months ago”.
    pub date_displayed: String,
    /// HTML content of the comment.
    pub html_content: String,
    /// List of the IDs of reply comments.
    pub children: Vec<i32>,
}

impl Comment {
    pub fn from_api_comment(comment: &hnapi::Comment, parent: Option<u32>) -> Vec<Comment> {
        let mut comments = vec![Comment {
            parent: parent.map(|parent| parent as i32),
            id: comment.id as i32,
            user: comment.user.clone(),
            date: comment.date.to_string(),
            date_displayed: comment.date_displayed.clone(),
            html_content: comment.html_content.clone(),
            children: comment
                .children
                .borrow()
                .iter()
                .map(|child| child.id as i32)
                .collect(),
        }];
        let mut children = Comment::flatten_tree(&comment.children.borrow(), Some(comment.id));
        comments.append(&mut children);
        comments
    }

    pub fn flatten_tree(comments: &Vec<Rc<hnapi::Comment>>, parent: Option<u32>) -> Vec<Comment> {
        comments
            .iter()
            .flat_map(|child| Comment::from_api_comment(child, parent))
            .collect()
    }
}

#[derive(GraphQLObject)]
/// Combination of a story, its HTML content, and its comments.
struct StoryWithDetails {
    /// Information about the story.
    pub story: Story,
    /// HTML content of the story.
    pub html_content: Option<String>,
    /// List of the comments posted on the story.
    pub comments: Vec<Comment>,
}

impl StoryWithDetails {
    pub fn from_api_story(details: &hnapi::StoryWithDetails) -> StoryWithDetails {
        StoryWithDetails {
            story: Story::from_api_story(&details.story),
            html_content: details.html_content.clone(),
            comments: Comment::flatten_tree(&details.comments, None),
        }
    }
}

#[derive(GraphQLObject)]
/// Information about a user.
pub struct User {
    /// User ID (their username).
    pub id: String,
    /// Creation date.
    pub created: String,
    /// Karma.
    pub karma: i32,
    /// About text (biography).
    pub about: String,
}

impl User {
    pub fn from_api_user(user: &hnapi::User) -> User {
        User {
            id: user.id.clone(),
            created: user.created.to_string(),
            karma: user.karma as i32,
            about: user.about.clone(),
        }
    }
}

#[derive(GraphQLObject)]
pub struct Auth {
    /// The auth token to put the in `Authorization` header in authenticated requets.
    pub token: String,
}

impl Auth {
    pub fn new(token: &str) -> Auth {
        Auth {
            token: token.to_string(),
        }
    }
}

#[derive(GraphQLInputObject)]
struct StoriesInListInput {
    /// The list to grab the stories from (default: top stories).
    list: Option<StoryList>,
    /// Page number, starting from 1 (default: first page). To be consistent
    /// with what HN’s website, will return the first page if lower than 1,
    /// and an empty page if greater than what HN accepts.
    page: Option<i32>,
}

#[derive(GraphQLInputObject)]
struct AuthInput {
    /// Username.
    username: String,
    /// Password.
    password: String,
}

#[derive(GraphQLInputObject)]
struct UpvoteStoryInput {
    /// The story ID.
    id: i32,
    /// The upvote auth token, found in the `upvoteAuth` field of the `stories` query.
    upvote_auth: String,
}

#[derive(Default, Clone)]
struct Context {
    pub auth_token: Option<String>,
}

struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// Get all the stories for a given list at a given page.
    async fn stories(
        context: &Context,
        input: StoriesInListInput,
    ) -> Result<Vec<StoryWithRank>, FieldError> {
        let stories = hnapi::stories_list(
            input.list.unwrap_or_default().to_api_story_list(),
            input.page.unwrap_or(1) as usize,
            &context.auth_token,
        )
        .await?;
        let mut ranks: Vec<usize> = stories.keys().copied().collect();
        ranks.sort();
        Ok(ranks
            .iter()
            .map(|rank| {
                let story = stories.get(rank).unwrap();
                StoryWithRank::from_api_story(*rank, &story)
            })
            .collect())
    }

    /// Get the details about a given story. Will return `null` for a non-existent story ID.
    async fn story(_context: &Context, id: i32) -> Result<Option<StoryWithDetails>, FieldError> {
        let story_with_details = hnapi::story_details(id as u32).await?;
        Ok(story_with_details.map(|details| StoryWithDetails::from_api_story(&details)))
    }

    /// Get the details about a given user. Will return `null` for a non-existent user ID.
    async fn user(_context: &Context, id: String) -> Result<Option<User>, FieldError> {
        let user = hnapi::user_details(&id).await?;
        Ok(user.map(|user| User::from_api_user(&user)))
    }

    /// Login and get the auth token used for next requests.
    async fn login(_context: &Context, input: AuthInput) -> Result<Auth, FieldError> {
        if let Some((token, _)) = hnapi::login(&input.username, &input.password).await? {
            Ok(Auth::new(&token))
        } else {
            Err(FieldError::new(
                "Invalid credentials.",
                graphql_value!(None),
            ))
        }
    }
}

struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    /// Upvote a story. You must be authenticated.
    async fn upvote_story(context: &Context, input: UpvoteStoryInput) -> Result<bool, FieldError> {
        if let Some(auth_token) = context.auth_token.as_ref() {
            if let Ok(ok) =
                hnapi::upvote_story(input.id as u32, &input.upvote_auth, auth_token).await
            {
                if ok {
                    Ok(true)
                } else {
                    Err(FieldError::new(
                        "Authentication error. You may need to login again.",
                        graphql_value!(None),
                    ))
                }
            } else {
                Err(FieldError::new(
                    "An error occurred while upvoting the story.",
                    graphql_value!(None),
                ))
            }
        } else {
            Err(FieldError::new(
                "You must be logged in to upvote a story. No auth token found in the headers.",
                graphql_value!(None),
            ))
        }
    }
}

type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, Mutation, EmptySubscription::<Context>::new());

    let state = warp::any().and(
        warp::header("authorization")
            .map(|auth_token| Context {
                auth_token: Some(auth_token),
            })
            .or(warp::any().map(|| Context { auth_token: None }))
            .unify(),
    );
    let graphql_filter = juniper_warp::make_graphql_filter(schema, state.boxed());

    let port = std::env::var("PORT")
        .map(|p| p.parse().expect("PORT must be a number"))
        .unwrap_or(8080);
    println!("Listening on port {}...", port);

    let graphiql_route = warp::get()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql", None));
    let graphql_route = warp::path("graphql").and(graphql_filter);
    let default_route = warp::path::end().map(|| warp::redirect(Uri::from_static("/graphiql")));

    warp::serve(graphiql_route.or(graphql_route).or(default_route))
        .run(([0, 0, 0, 0], port))
        .await
}
