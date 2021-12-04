# A GraphQL API for HackerNews

[![Rust](https://github.com/scastiel/hn/actions/workflows/rust.yml/badge.svg)](https://github.com/scastiel/hn/actions/workflows/rust.yml)

HackerNews’ [official API](https://github.com/HackerNews/API) is nice, but has two major drawbacks:

- it only offers _read_ operations (so not possible to upvote a story or post anything);
- it doesn’t offer an efficient way to list all the comments posted on a story (you have to perform one request per comment ID).

This is an attempt to create a clean and documented [GraphQL](https://graphql.org/) API, which would ultimately offer the same features as the HackerNews website itself. The goal is to be able to create alternative frontends for HackerNews: a web or mobile application, a [command-line interface](https://github.com/scastiel/hn/tree/main/cli)… sky is the limit!

This GraphQL API uses [a library](https://github.com/scastiel/hn/tree/main/api) performing scraping on the HackerNews website. Usually, scrapping a website would be a terrible idea and could stop working any day; but the good thing with HackerNews is that its design is never updated ;)

## Screenshots

<table>
  <tr>
    <td><img src="https://github.com/scastiel/hn/blob/main/graphql/graphiql_screenshot01.png?raw=true"/></td>
    <td><img src="https://github.com/scastiel/hn/blob/main/graphql/graphiql_screenshot02.png?raw=true"/></td>
  </tr>
</table>

## Usage

I deployed the server on Heroku. You can play with [GraphiQL](https://hackernews-graphql-api.herokuapp.com/graphiql) there and use it to view the documentation of the available queries and mutations. Note that this version of GraphiQL doesn’t support adding HTTP headers easilly, so you won’t be able to run authenticated requests.

As an alternative, other IDEs exists to use GraphQL API with a better user experience, such as:

- [GraphQL Playground](https://github.com/graphql/graphql-playground)
- [GraphiQL App](https://github.com/skevy/graphiql-app)

The API endpoint is `https://hackernews-graphql-api.herokuapp.com/graphql`.

You can also clone this repository to deploy your own version of the server :)

## Features

- [x] List stories
- [x] Get story details
- [x] Get story comments
- [x] Get user information
- [x] Login and get auth token
- [x] Upvote a story

### To be implemented

- [ ] Unvote a story
- [ ] Upvote and unvote a comment
- [ ] Post/edit a story
- [ ] Post/edit a comment
- [ ] Get more information about a user: submissions, comments…

## Examples

One of the advantage of GraphQL APIs is that they are self-documented, so you can use the deployed [GraphiQL](https://hackernews-graphql-api.herokuapp.com/graphiql) to get the full documentation. But here are some examples of what you can do with it:

### List stories

With the `story` query, you can display the 30 stories visible for a given list on a given page.

Note that if passing the auth token in the `Authorization` header, the `upvoteAuth` field with contain the token to pass to the `upvote` mutation. Otherwise the field will be null.

<details>
<summary>Query</summary>

```graphql
query GetStories($input: StoriesInListInput!) {
  stories(input: $input) {
    rank
    story {
      id
      title
      url
      urlDisplayed
      upvoteAuth
      user
      score
      date
      dateDisplayed
      commentCount
    }
  }
}
```

</details>

<details>
<summary>Variables</summary>

```json
{
  "input": {
    "list": "NEWS",
    "page": 1
  }
}
```

</details>

<details>
<summary>Headers (optional)</summary>

```
Authorization: youruser&thisisyourauthtoken
```

</details>

<details>
<summary>Result</summary>

```json
{
  "data": {
    "stories": [
      {
        "rank": 1,
        "story": {
          "id": 29432276,
          "title": "U.S. State Department phones hacked with Israeli company spyware",
          "url": "https://www.reuters.com/technology/exclusive-us-state-department-phones-hacked-with-israeli-company-spyware-sources-2021-12-03/",
          "urlDisplayed": "reuters.com",
          "upvoteAuth": null,
          "user": "amadeuspagel",
          "score": 821,
          "date": "2021-12-03 17:05:27 UTC",
          "dateDisplayed": "9 hours ago",
          "commentCount": 391
        }
      }
      // ...
    ]
  }
}
```

</details>

### Show details and comments

With the `story` query, you can get the details about a story, including its HTML content (for text stories) and its comments.

The comments are returned in a flat list, each one containing its parent ID (if any), and the list of its children IDs.

<details>
<summary>Query</summary>

```graphql
query StoryDetails($id: Int!) {
  story(id: $id) {
    story {
      id
      title
      url
      urlDisplayed
      upvoteAuth
      user
      score
      date
      dateDisplayed
      commentCount
    }
    htmlContent
    comments {
      parent
      id
      user
      date
      dateDisplayed
      htmlContent
      children
    }
  }
}
```

</details>

<details>
<summary>Variables</summary>

```json
{
  "id": 29432276
}
```

</details>

<details>
<summary>Result</summary>

```json
{
  "data": {
    "story": {
      "story": {
        "id": 29432276,
        "title": "U.S. State Department phones hacked with Israeli company spyware",
        "url": "https://www.reuters.com/technology/exclusive-us-state-department-phones-hacked-with-israeli-company-spyware-sources-2021-12-03/",
        "urlDisplayed": "reuters.com",
        "upvoteAuth": null,
        "user": "amadeuspagel",
        "score": 823,
        "date": "2021-12-03 17:05:27 UTC",
        "dateDisplayed": "9 hours ago",
        "commentCount": 393
      },
      "htmlContent": null,
      "comments": [
        {
          "parent": null,
          "id": 29434071,
          "user": "markus_zhang",
          "date": "2021-12-03 19:20:52 UTC",
          "dateDisplayed": "7 hours ago",
          "htmlContent": "This is a typical \"shadow government\" symptom. You have forces working within the government that 1) have their own agendas; 2) have connection to international communities, usually military-intelligence ones; 3) have almost zero regulation; 4) even many high ranking government officials don't know about them because they are brotherhood-like closed circles.<p>This reminds me of Operation Gladio or Propaganda Due but domestic. Same playbook, different players.\n              </p>",
          "children": [29437443, 29436527, 29434655, 29434410, 29435635]
        },
        {
          "parent": 29434071,
          "id": 29437443,
          "user": "refurb",
          "date": "2021-12-04 00:36:04 UTC",
          "dateDisplayed": "1 hour ago",
          "htmlContent": "“Shadow government” = “Deep state”?",
          "children": []
        }
        // ...
      ]
    }
  }
}
```

</details>

### Get details about a user

With the `user` query, you can get the information about a user.

<details>
<summary>Query</summary>

```graphql
query GetUser($username: String!) {
  user(id: $username) {
    id
    created
    karma
    about
  }
}
```

</details>

<details>
<summary>Variables</summary>

```json
{
  "username": "scastiel"
}
```

</details>

<details>
<summary>Result</summary>

```json
{
  "data": {
    "user": {
      "id": "scastiel",
      "created": "2019-02-16",
      "karma": 513,
      "about": "Find me on Twitter: @scastiel"
    }
  }
}
```

</details>

### Login and get an auth token

Using the `login` query, you can get an auth token that you can then pass as the `Authorization` header when running mutations that need to be authenticated.

<details>
<summary>Query</summary>

```graphql
query Login($input: AuthInput!) {
  login(input: $input) {
    token
  }
}
```

</details>

<details>
<summary>Variables</summary>

```json
{
  "input": {
    "username": "yourusername",
    "password": "yourpassword"
  }
}
```

</details>

<details>
<summary>Result</summary>

```json
{
  "data": {
    "login": {
      "token": "yourauthtoken"
    }
  }
}
```

</details>

### Upvote a story

When authenticated, you can use the `upvote_story` mutation to upvote a story. You will need:

- to pass as variable the _upload auth token_ that you get from the `uploadAuth` field in the `query` request,
- to pass in the `Authorization` header your auth token returned by the `login` query.

<details>
<summary>Query</summary>

```graphql
mutation Upvote($input: UpvoteStoryInput!) {
  upvoteStory(input: $input)
}
```

</details>

<details>
<summary>Variables</summary>

```json
{
  "input": {
    "id": 29425735,
    "upvoteAuth": "theuploadauthtoken"
  }
}
```

</details>

<details>
<summary>Headers</summary>

```
Authorization: youruser&thisisyourauthtoken
```

</details>

<details>
<summary>Result</summary>

```json
{
  "data": {
    "upvoteStory": true
  }
}
```

</details>

## License

MIT, see [LICENSE](https://github.com/scastiel/hn/blob/main/api/LICENSE).
