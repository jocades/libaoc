# libaoc

Advent of Code CLI and utilities.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/libaoc.svg
[crates-url]: https://crates.io/crates/libaoc
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/jocades/libaoc/blob/main/LICENSE

## CLI

- `Get` the puzzle's questions and input. The **contents** are **cached** after every request / submission.
- `Submit` the answer in a couple of key presses.
- `View` the puzzle from the terminal or editor in **markdown** format.

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
structure. If the puzzle's part is omitted, it will smartly choose the correct
one. Almost all commands can be shortened in this way.

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

A `Rust client` for the Advent of Code API. It needs aoc's session token to
verify the user.

### Install

```sh
cargo add libaoc
```

### Example

```rs
use libaoc:::Client;

let token = "53616c...";
let client = Client::new(token)?;
```

Get a puzzle from the cache or by scraping the website

```rs
let id = (2024, 1);
let puzzle = client.get_puzzle(&id)?;
prinln!("q1 {}", puzzle.q1);
prinln!("a1 {}", puzzle.a1);
```

Submit an answer.

```rs
let id = (2024, 1);
let part = 2;
client.submit(&id, part, "answer")?;
```
