use nom::{
    character::complete::{anychar, not_line_ending},
    multi::many1,
    IResult,
};
use nom_span::Spanned;

extern crate nom;
extern crate nom_span;

type Span<'a> = Spanned<&'a str>;

#[test]
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

// What is important in this test is the fact that it prevents a regression for the compare
// implementation.
// `not_line_ending` requires that `T: Compare<&'static str>`, and it wasn't working before
#[test]
fn can_compare_with_different_type() {
    pub fn until_newline(input: Span) -> IResult<Span, &str> {
        let (rest, comment_str) = not_line_ending(input)?;
        Ok((rest, *comment_str))
    }
    let utf8 = Span::new("test\n", true);

    let (utf8_after, _) = until_newline(utf8).unwrap();

    assert_eq!(*utf8_after, "\n");
}
