# `hn-cli`, a command-line tool to read [HackerNews](https://news.ycombinator.com)

[![Crates.io](https://img.shields.io/crates/v/hn-cli)](https://crates.io/crates/hn-cli)
[![Rust](https://github.com/scastiel/hn/actions/workflows/rust.yml/badge.svg)](https://github.com/scastiel/hn/actions/workflows/rust.yml)

## Screenshots

<table>
  <tr>
    <td><img src="https://github.com/scastiel/hn/blob/main/cli/screenshot01.png?raw=true"/></td>
    <td><img src="https://github.com/scastiel/hn/blob/main/cli/screenshot02.png?raw=true"/></td>
  </tr>
</table>

## Installation

You’ll need first to [install the Rust toolchain](https://rustup.rs/), then: `cargo install hn-cli`

## Usage

List stories (add `-p3` or `--page 3` to display the third page):

- Top stories: `hn` or `hn top` or `hn t`
- New stories: `hn new` or `hn n`
- Best stories: `hn best` or `hn b`
- Show HN stories: `hn show` or `hn s`
- Ask HN stories: `hn ask` or `hn a`
- Job stories: `hn job` or `hn j`

After listing stories, note the index of the story you are interested in (let’s suppose it is `5`), then:

- Show story details and comments: `hn details 5` or `hn d 5`
- Open story link in your browser: `hn open 5` or `hn o 5`

You can also display the details about a user with `hn user the_user_name` or `hn u the_user`.

**Note:** information is obtained by scraping the HackerNews website. The reason this crate does not use the [official API](https://github.com/HackerNews/API) is that it does not provide a convenient way to get all the comments for a given story.

## License

MIT, see [LICENSE](https://github.com/scastiel/hn/blob/main/api/LICENSE).
