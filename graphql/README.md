# A GraphQL API to get HackerNews stories

[![Rust](https://github.com/scastiel/hn/actions/workflows/rust.yml/badge.svg)](https://github.com/scastiel/hn/actions/workflows/rust.yml)

## Screenshots

<table>
  <tr>
    <td><img src="https://github.com/scastiel/hn/blob/main/graphql/graphiql_screenshot01.png?raw=true"/></td>
    <td><img src="https://github.com/scastiel/hn/blob/main/graphql/graphiql_screenshot02.png?raw=true"/></td>
  </tr>
</table>

## Usage

I deployed the server on Heroku. You can play with [GraphiQL](https://hackernews-graphql-api.herokuapp.com/graphiql) and use it to view the documentation of the available queries. The endpoint is `https://hackernews-graphql-api.herokuapp.com/graphql`.

Or you can clone this repository to deploy your own version of the server :)

**Note:** information is obtained by scraping the HackerNews website. The reason this API does not use the [official API](https://github.com/HackerNews/API) is that it does not provide a convenient way to get all the comments for a given story, and only allows read operations.

## License

MIT, see [LICENSE](https://github.com/scastiel/hn/blob/main/api/LICENSE).
