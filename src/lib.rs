//! An opinionated parser combinator library with a focus on fully abortable parsing and error handling.
//!
//! # Example
//!
//! ```
//! #[macro_use]
//! extern crate abortable_parser;
//! use abortable_parser::iter::StrIter;
//! use abortable_parser::{Result, eoi, ascii_ws};
//!
//! make_fn!(proto<StrIter, &str>,
//!     do_each!(
//!         proto => until!(text_token!("://")),
//!         _ => must!(text_token!("://")),
//!         (proto)
//!     )
//! );
//!
//! make_fn!(domain<StrIter, &str>,
//!     until!(either!(
//!         discard!(text_token!("/")),
//!         discard!(ascii_ws),
//!         eoi))
//! );
//!
//! make_fn!(path<StrIter, &str>,
//!      until!(either!(discard!(ascii_ws), eoi))
//! );
//!
//! make_fn!(url<StrIter, (Option<&str>, Option<&str>, &str)>,
//!     do_each!(
//!         protocol => optional!(proto),
//!         domain => optional!(domain),
//!         path => path,
//!         (protocol, domain, path)
//!     )
//! );
//!
//! # fn main() {
//! let iter = StrIter::new("http://example.com/some/path ");
//! let result = url(iter);
//! assert!(result.is_complete());
//! if let Result::Complete(_, (proto, domain, path)) = result {
//!     assert!(proto.is_some());
//!     assert!(domain.is_some());
//!     assert_eq!(path, "/some/path");
//! }
//! # }
//! ```
use std::fmt::Display;
use std::iter::Iterator;

/// A trait for types that can have an offset as a count of processed items.
pub trait Offsetable {
    fn get_offset(&self) -> usize;
}

impl Offsetable for usize {
    fn get_offset(&self) -> usize {
        return *self;
    }
}

pub enum SpanRange {
    Range(std::ops::Range<usize>),
    RangeTo(std::ops::RangeTo<usize>),
    RangeFrom(std::ops::RangeFrom<usize>),
    RangeFull(std::ops::RangeFull),
}

// An input that can provide a span of a range of the input.
pub trait Span<O> {
    fn span(&self, idx: SpanRange) -> O;
}

/// A Cloneable Iterator that can report an offset as a count of processed Items.
pub trait InputIter: Iterator + Clone + Offsetable {}

/// The custom error type for use in `Result::{Fail, Abort}`.
/// Stores a wrapped err that must implement Display as well as an offset and
/// an optional cause.
#[derive(Debug)]
pub struct Error {
    msg: String,
    offset: usize,
    cause: Option<Box<Error>>,
}

impl Error {
    /// Constructs a new Error with an offset and no cause.
    pub fn new<S, M>(msg: M, offset: &S) -> Self
    where
        S: Offsetable,
        M: Into<String>,
    {
        Error {
            msg: msg.into(),
            offset: offset.get_offset(),
            cause: None,
        }
    }

    /// Constructs a new Error with an offset and a cause.
    pub fn caused_by<S, M>(msg: M, offset: &S, cause: Self) -> Self
    where
        S: Offsetable,
        M: Into<String>,
    {
        Error {
            msg: msg.into(),
            offset: offset.get_offset(),
            cause: Some(Box::new(cause)),
        }
    }

    /// Returns the contained err.
    pub fn get_msg<'a>(&'a self) -> &'a str {
        &self.msg
    }

    /// Returns `Some(cause)` if there is one, None otherwise.
    pub fn get_cause<'a>(&'a self) -> Option<&'a Error> {
        match self.cause {
            Some(ref cause) => Some(cause),
            None => None,
        }
    }

    // Returns the offset at which this Error happened.
    pub fn get_offset(&self) -> usize {
        self.offset
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        try!(write!(f, "{}", self.msg));
        match self.cause {
            Some(ref c) => write!(f, "\n\tCaused By:{}", c),
            None => Ok(()),
        }
    }
}

/// The result of a parsing attempt.
#[derive(Debug)]
pub enum Result<I: InputIter, O> {
    /// Complete represents a successful match.
    Complete(I, O),
    /// Incomplete indicates input ended before a match could be completed.
    /// It contains the offset at which the input ended before a match could be completed.
    Incomplete(usize),
    /// Fail represents a failed match.
    Fail(Error),
    /// Abort represents a match failure that the parser cannot recover from.
    Abort(Error),
}

impl<I: InputIter, O> Result<I, O> {
    /// Returns true if the Result is Complete.
    pub fn is_complete(&self) -> bool {
        if let &Result::Complete(_, _) = self {
            return true;
        }
        return false;
    }

    /// Returns true if the Result is Incomoplete.
    pub fn is_incomplete(&self) -> bool {
        if let &Result::Incomplete(_) = self {
            return true;
        }
        return false;
    }

    /// Returns true if the Result is Fail.
    pub fn is_fail(&self) -> bool {
        if let &Result::Fail(_) = self {
            return true;
        }
        return false;
    }

    /// Returns true if the Result is Abort.
    pub fn is_abort(&self) -> bool {
        if let &Result::Abort(_) = self {
            return true;
        }
        return false;
    }
}

pub use combinators::*;
pub use iter::SliceIter;

#[macro_use]
pub mod combinators;
pub mod iter;

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod test;
