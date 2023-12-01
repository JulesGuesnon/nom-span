use nom::{character::complete::anychar, multi::many1, IResult};
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
