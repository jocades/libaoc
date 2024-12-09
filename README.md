# libaoc

Advent of Code CLI and utilities.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/libaoc.svg
[crates-url]: https://crates.io/crates/libaoc
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/jocades/libaoc/blob/main/LICENSE

## CLI

Handle **puzzle retrieval**, **viewing** and **submission** from the **command line**.

- `Get` the puzzle's questions and input. The **contents** are **cached** after every request / submission.
- `Submit` the answer in a couple of key presses.
- `View` the puzzle from the terminal or editor in **markdown** format.

> [!WARNING]
> An `AOC_AUTH_TOKEN` environment variable is necessary for user validation. See [here](#session-token) to find out how to get your token.

### Install

```sh
cargo install libaoc
```

### Example

Retrieve the puzzle's questions and answers for a certain day and year.

```sh
aoc get -y 2024 -d -1
```

Submit an answer for a specific puzzle and part.

```sh
aoc submit -y 2024 -d -1 -p 2 "answer"
Correct!
```

If the day or year is omitted, it will be derived from the current directory's
structure. If the puzzle's part is omitted, it will smartly be chosen from the
puzzle state. Almost all commands can be shortened in this way.

```sh
pwd
/Users/j0rdi/aoc/2024/d01

aoc submit "answer"
Correct!
```

Take a look at the help.

```sh
aoc --help
```

<details>
<summary>Show output.</summary>

```sh
Usage: aoc [OPTIONS] <COMMAND>

Commands:
  get
  submit
  view
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
  -h, --help     Print help
  -V, --version  Print version
```

</details>

## API

A **Rust client** for the Advent of Code API.

### Install

```sh
cargo add libaoc
```

### Example

Use the client to scrape a puzzle from the website. The [session token](#session-token) is
required to verify the user.

```rs
use libaoc:::Client;

let token = "53616c...";
let client = Client::new(token)?;
let puzzle = client.scrape_puzzle((2024, 1));
```

Get a puzzle from the cache or by scraping the website if the puzzle is not present.

```rs
let id = (2024, 1);
let puzzle = client.get_puzzle(&id)?;
prinln!("q1 {}", puzzle.q1);
prinln!("a1 {}", puzzle.a1);
```

Submit an answer. When using the CLI the id and part can be automatically
determined when omitted.

```rs
let id = (2024, 1);
let part = 2;
client.submit(&id, part, "answer")?;
```

Refresh the in cache puzzle by re-scraping the website. This is done after a
successful submission of part one to retrieve part two automatically.

```rs
let puzzle = client.refresh_puzzle((2024, 1))
```

## Session token

To correctly set your `AOC_AUTH_TOKEN` environment variable, find the `cookie`
field in the request headers used when requesting a page, `https://adventofcode.com/2015/day/1`
for example, you can do so by opening the `network panel` in your browser and
check for the field `cookie` in the request headers of the current page. You
can also right click on the request and copy the cURL command used.

Remember that you `must be logged in` for your session cookie to be present in your request headers.
