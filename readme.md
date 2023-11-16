# Nom span

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
