#![deny(clippy::all)]
#![deny(clippy::pedantic)]
//! This library expose `Spanned`, a struct that will wraps your input and allow you to keep track of the line number, the column number and the byte offset
//!
//! ## How to use it?
//!
//! Here is a basic example of how to create the input and how to retrieve all the informations you need.
//!
//! ```ignore
//! use nom_span::Spanned;
//!
//! type Span<'a> = Spanned<&'a str>;
//!
//! fn main() {
//!     let span = Span::new(
//!       r#"{"hello": "world 🙌"}"#,
//!       // Supporting UTF-8
//!       true
//!     );
//!
//!     assert_eq!(span.line(), 1);
//!     assert_eq!(span.col(), 1);
//!     assert_eq!(span.byte_offset(), 0);
//! }
//! ```
//!
//! You can notice that supporting UTF-8 is optional. The reason is that UTF-8 strings need to be handled in a different way than pure ASCII strings, and thus, there can be a performance gap with UTF-8 support (see the benchmark below)
//!
//! ### UTF-8 and ASCII comparison
//!
//! A UTF-8 char can be made of 1 to 4 bytes, so counting it the ASCII way would result in counting each byte of the UTF-8 char, and will result in unexpected column number:
//!
//! ```ignore
//! use nom_span::Spanned;
//!
//! type Span<'a> = Spanned<&'a str>;
//!
//! fn utf8_vs_ascii() {
//!     let utf8 = Span::new("🙌", true);
//!     let ascii = Span::new("🙌", false);
//!
//!     let utf8_after: IResult<Span<'_>, Vec<char>> = many1(anychar)(utf8);
//!     let ascii_after: IResult<Span<'_>, Vec<char>> = many1(anychar)(ascii);
//!
//!     let (utf8_after, _) = utf8_after.unwrap();
//!     let (ascii_after, _) = ascii_after.unwrap();
//!
//!     assert_eq!(utf8_after.col(), 2);
//!     assert_eq!(ascii_after.col(), 5);
//! }
//!
//! ```
//!
//! ## What about [nom_locate](https://github.com/fflorent/nom_locate)?
//!
//! I was initially using [nom_locate](https://github.com/fflorent/nom_locate), but I faced some huge performance issue while building a [json parser](https://github.com/julesguesnon/spanned-json-parser), so I decided to implement my own input. I basically cloned [nom_locate](https://github.com/fflorent/nom_locate) and modified the counting function that was causing the performance issue. So thanks a lot for this awesome crate and please go add a star to it!
//!
//! ### What's the difference with [nom_locate](https://github.com/fflorent/nom_locate)?
//!
//! [nom_locate](https://github.com/fflorent/nom_locate) is recounting all the chars of your entire input (even if you already consumed it) when you're calling `get_column`. If you're calling `get_column` every char, runtime would be: `O(N^2)`
//! With this crate, it's counting lines and columns everytime you're consuming your input. If you're calling `col` every char, runtime would be: `O(2N)`
//!
//! So if you're planning to get the column only a few times, for example, only when an error occur, it may be better to use [nom_locate](https://github.com/fflorent/nom_locate), but if you need it quite often, this crate should be better.

use bytecount::num_chars;
use memchr::Memchr;
use nom::{
    AsBytes, Compare, Err, ExtendInto, FindSubstring, FindToken, InputIter, InputLength, InputTake,
    InputTakeAtPosition, Offset, ParseTo, Slice,
};
use std::{
    ops::{RangeFrom, RangeTo},
    str::FromStr,
};

extern crate bytecount;
extern crate memchr;
extern crate nom;

/// You can wrap your input in this struct with [`Spanned::new`]
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    data: T,
    line: usize,
    col: usize,
    offset: usize,
    handle_utf8: bool,
}

impl<T> Spanned<T> {
    pub fn new(data: T, handle_utf8: bool) -> Self {
        Self {
            data,
            line: 1,
            col: 1,
            offset: 0,
            handle_utf8,
        }
    }

    /// Get the current line number
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get the current column number
    pub fn col(&self) -> usize {
        self.col
    }

    /// Get the current byte offset
    pub fn byte_offset(&self) -> usize {
        self.offset
    }

    /// Get the current data in the span
    pub fn data(&self) -> &T {
        &self.data
    }
}

impl<T> core::ops::Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, U> core::convert::AsRef<U> for Spanned<&T>
where
    T: ?Sized + core::convert::AsRef<U>,
    U: ?Sized,
{
    fn as_ref(&self) -> &U {
        self.data.as_ref()
    }
}

impl<T> AsBytes for Spanned<T>
where
    T: AsBytes,
{
    fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }
}

impl<T, Comp> Compare<Comp> for Spanned<T>
where
    T: Compare<Comp>,
{
    fn compare(&self, t: Comp) -> nom::CompareResult {
        self.data.compare(t)
    }

    fn compare_no_case(&self, t: Comp) -> nom::CompareResult {
        self.data.compare_no_case(t)
    }
}

impl<T> ExtendInto for Spanned<T>
where
    T: ExtendInto,
{
    type Item = T::Item;

    type Extender = T::Extender;

    fn new_builder(&self) -> Self::Extender {
        self.data.new_builder()
    }

    fn extend_into(&self, acc: &mut Self::Extender) {
        self.data.extend_into(acc);
    }
}

impl<T> FindSubstring<T> for Spanned<T>
where
    T: FindSubstring<T>,
{
    fn find_substring(&self, substr: T) -> Option<usize> {
        self.data.find_substring(substr)
    }
}

impl<T, Token> FindToken<Token> for Spanned<T>
where
    T: FindToken<Token>,
{
    fn find_token(&self, token: Token) -> bool {
        self.data.find_token(token)
    }
}

impl<T> InputIter for Spanned<T>
where
    T: InputIter,
{
    type Item = T::Item;

    type Iter = T::Iter;

    type IterElem = T::IterElem;

    fn iter_indices(&self) -> Self::Iter {
        self.data.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.data.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.data.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.data.slice_index(count)
    }
}

impl<T> InputLength for Spanned<T>
where
    T: InputLength,
{
    fn input_len(&self) -> usize {
        self.data.input_len()
    }
}

impl<T> InputTake for Spanned<T>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<T> InputTakeAtPosition for Spanned<T>
where
    T: InputTakeAtPosition + InputLength + InputIter,
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Clone,
{
    type Item = <T as InputIter>::Item;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.data.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        _e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.data.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.split_at_position(predicate) {
            Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
            res => res,
        }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.data.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.data.input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl<T> Offset for Spanned<T>
where
    T: Offset,
{
    fn offset(&self, second: &Self) -> usize {
        self.data.offset(&second.data)
    }
}

impl<T, R: FromStr> ParseTo<R> for Spanned<T>
where
    T: ParseTo<R>,
{
    fn parse_to(&self) -> Option<R> {
        self.data.parse_to()
    }
}

impl<T, R> Slice<R> for Spanned<T>
where
    T: Slice<R> + Offset + AsBytes + Slice<RangeTo<usize>>,
{
    fn slice(&self, range: R) -> Self {
        let next_data = self.data.slice(range);

        let offset = self.data.offset(&next_data);

        let old_data = self.data.slice(..offset);

        if offset == 0 {
            return Self {
                data: next_data,
                line: self.line,
                col: self.col,
                offset: self.offset,
                handle_utf8: self.handle_utf8,
            };
        }

        let new_line_iter = Memchr::new(b'\n', old_data.as_bytes());

        let mut lines_to_add = 0;
        let mut last_index = None;
        for i in new_line_iter {
            lines_to_add += 1;
            last_index = Some(i);
        }
        let last_index = last_index.map_or(0, |v| v + 1);

        let col = if self.handle_utf8 {
            num_chars(old_data.as_bytes().slice(last_index..))
        } else {
            old_data.as_bytes().len() - last_index
        };

        Self {
            data: next_data,
            line: self.line + lines_to_add,
            col: if lines_to_add == 0 {
                self.col + col
            } else {
                // When going to a new line, char starts at 1
                col + 1
            },
            offset: self.offset + offset,
            handle_utf8: self.handle_utf8,
        }
    }
}
