# Nom Span &emsp; [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/actions/workflow/status/julesguesnon/nom-span/rust.yml?branch=main
[actions]: https://github.com/julesguesnon/nom-span/actions?query=branch%3Amain
[crates.io]: https://crates.io/crates/nom-span
[Latest Version]: https://img.shields.io/crates/v/nom-span.svg

This library expose `Spanned`, a struct that will wraps your input and allow you to keep track of the line number, the column number and the byte offset

## How to use it?

Here is a basic example of how to create the input and how to retrieve all the informations you need.

```rust
use nom_span::Spanned;

type Span<'a> = Spanned<&'a str>;

fn main() {
    let span = Span::new(
      r#"{"hello": "world 🙌"}"#,
      // Supporting UTF-8
      true
    );

    assert_eq!(span.line(), 1);
    assert_eq!(span.col(), 1);
    assert_eq!(span.byte_offset(), 0);
}
```

You can notice that supporting UTF-8 is optional. The reason is that UTF-8 strings need to be handled in a different way than pure ASCII strings, and thus, there can be a performance gap with UTF-8 support (see the benchmark below)

### UTF-8 and ASCII comparison

A UTF-8 char can be made of 1 to 4 bytes, so counting it the ASCII way would result in counting each byte of the UTF-8 char, and will result in unexpected column number:

```rust
use nom_span::Spanned;

type Span<'a> = Spanned<&'a str>;

fn utf8_vs_ascii() {
    let utf8 = Span::new("🙌", true);
    let ascii = Span::new("🙌", false);

    let utf8_after: IResult<Span<'_>, Vec<char>> = many1(anychar)(utf8);
    let ascii_after: IResult<Span<'_>, Vec<char>> = many1(anychar)(ascii);

    let (utf8_after, _) = utf8_after.unwrap();
    let (ascii_after, _) = ascii_after.unwrap();

    assert_eq!(utf8_after.col(), 2);
    assert_eq!(ascii_after.col(), 5);
}

```

## What about [nom_locate](https://github.com/fflorent/nom_locate)?

I was initially using nom_locate, but I faced some huge performance issue while building a [json parser](https://github.com/julesguesnon/spanned-json-parser), so I decided to implement my own input. I basically cloned nom_locate and modified the counting function that was causing the performance issue. So thanks a lot for this awesome crate and please go add a star to it!

### What's the difference with [nom_locate](https://github.com/fflorent/nom_locate)?

nom_locate is recounting all the chars of your entire input (even if you already consumed it) when you're calling `get_column`. If you're calling `get_column` every char, runtime would be: `O(N^2)`
With this crate, it's counting lines and columns everytime you're consuming your input. If you're calling `col` every char, runtime would be: `O(2N)`

So if you're planning to get the column only a few times, for example, only when an error occur, it may be better to use nom_locate, but if you need it quite often, this crate should be better.
