//! Contains matchers for matching specific patterns or tokens.
use super::{InputIter, Result, Error};

use std::fmt::{Debug, Display};

/// Convenience macro for looking for a specific text token in a byte input stream.
/// 
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// use std::convert::From;
/// # fn main() {
/// let iter: iter::SliceIter<u8> = "foo bar".into();
/// let tok = text_token!(iter, "foo");
/// # assert!(tok.is_complete());
/// if let Result::Complete(i, o) = tok {
///     assert_eq!(i.get_offset(), 3);
///     assert_eq!(o, "foo");
/// }
/// # }
/// ```
#[macro_export]
macro_rules! text_token {
    ($i:expr, $e:expr) => {{
        use $crate::Error;
        use $crate::Result;
        let mut _i = $i.clone();
        let mut count = 0;
        for expected in $e.bytes() {
            let item = match _i.next() {
                Some(item) => item,
                None => break,
            };
            if item == &expected {
                count += 1;
            }
        }
        if count == $e.len() {
            Result::Complete(_i.clone(), $e)
        } else {
            Result::Fail(Error::new(
                format!("Expected {} but didn't get it.", $e),
                &$i,
            ))
        }
    }};
}

/// Consumes an input until it reaches the term combinator matches.
///
/// If the term never matches then returns incomplete.
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// use std::convert::From;
/// # fn main() {
/// let iter: iter::SliceIter<u8> = "foo;".into();
/// let tok = until!(iter, text_token!(";"));
/// # assert!(tok.is_complete());
/// if let Result::Complete(i, o) = tok {
///     assert_eq!(i.get_offset(), 3);
/// }
/// # }
/// ```
#[macro_export]
macro_rules! until {
    ($i:expr, $term:ident!( $( $args:tt )* ) ) => {{
        use $crate::{Result, Offsetable};
        let mut acc = Vec::new();
        let mut _i = $i.clone();
        let pfn = || {
            loop {
                match $term!(_i.clone(), $($args)*) {
                    Result::Complete(_, _) => return Result::Complete(_i, acc),
                    Result::Abort(e) => return Result::Abort(e),
                    Result::Incomplete(offset) => return Result::Incomplete(offset),
                    Result::Fail(_) => {
                        // noop
                    }
                }
                let item = match _i.next() {
                    Some(it) => it,
                    None => return Result::Incomplete(_i.get_offset()),
                };
                acc.push(item);
            }
        };
        pfn()
    }};

    ($i:expr, $term:ident) => {
        consume_until!($i, run!($term))
    };
}

/// Maps a Result of type Vec<&u8> to a Result of type String.
pub fn must_string<'a, I, E>(matched: Result<I, Vec<&'a u8>, E>, msg: E) -> Result<I, String, E>
where
    I: InputIter<Item=&'a u8>,
    E: Debug + Display,
{
        match matched {
            Result::Complete(i, mut o) => {
                let new_string = String::from_utf8(o.drain(0..).map(|b| *b).collect());
                match new_string {
                    Ok(s) => Result::Complete(i, s),
                    Err(_) => Result::Abort(Error::new(msg, &i)),
                }
            },
            Result::Incomplete(offset) => Result::Incomplete(offset),
            Result::Abort(e) => Result::Abort(e),
            Result::Fail(e) => Result::Fail(e),
        }
}