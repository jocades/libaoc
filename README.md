# libaoc

Advent of Code CLI and utilities.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/libaoc.svg
[crates-url]: https://crates.io/crates/libaoc
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/jocades/libaoc/blob/main/LICENSE

## Install

```sh
cargo install libaoc
```

## CLI

- `Get` the puzzle's questions and input. The **contents** are **cached** after every request / submission.
- `Read` the puzzle from the terminal or editor in **markdown** format.
- `Submit` the answer from in a couple of key presses.


Retrieve the puzzle's questions and answers for a certain day and year.


```sh
aoc get -y 2024 -d -1
```

Submit an answer for a specific puzzle.

```sh
aoc submit -y 2024 -d -1 "answer"
Correct!
```

If the day or year is omitted, it will be derived from the current directory's
structure.

```sh
pwd
/Users/j0rdi/aoc/2024/d01

aoc submit "answer"
Correct!
```

## API
