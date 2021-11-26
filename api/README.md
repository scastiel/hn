# `hnapi`, a crate to query stories from [HackerNews](https://news.ycombinator.com/).

[![Crates.io](https://img.shields.io/crates/v/hnapi)](https://crates.io/crates/hnapi)
[![Rust](https://github.com/scastiel/hn/actions/workflows/rust.yml/badge.svg)](https://github.com/scastiel/hn/actions/workflows/rust.yml)

For now, it supports three operations:

- list stories using [`stories_list`](https://docs.rs/hnapi/latest/hnapi/fn.stories_list.html)
- get details and comments for a story using [`story_details`](https://docs.rs/hnapi/latest/hnapi/fn.story_details.html)
- get details about a user using [`user_details`](https://docs.rs/hnapi/latest/hnapi/fn.user_details.html)

Refer to their respective documentations to see usage examples.

**Note:** information is obtained by scraping the HackerNews website. The reason this crate does not use the [official API](https://github.com/HackerNews/API) is that it does not provide a convenient way to get all the comments for a given story.

## License

MIT, see [LICENSE](https://github.com/scastiel/hn/blob/main/api/LICENSE).
