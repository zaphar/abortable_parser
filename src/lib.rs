// Copyright 2017 Jeremy Wall <jeremy@marzhillstudios.com>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

//! An opinionated parser combinator library with a focus on fully abortable parsing and
//! easy error handling.
//!
//! The approach to macro composition is heavily inspired by nom. It focuses on a simple
//! API for combinators, and easy error handling.
//!
//! We have a number of macros that assist in the gneration or handling of each type
//! of error.
//!
//! # Simple parsing of a url.
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
//!     do_each!(
//!         // domains do not start with a slash
//!         _ => peek!(not!(text_token!("/"))),
//!         domain => until!(either!(
//!             discard!(text_token!("/")),
//!             discard!(ascii_ws),
//!             eoi)),
//!         (domain)
//!     )
//! );
//!
//! make_fn!(path<StrIter, &str>,
//!      until!(either!(discard!(ascii_ws), eoi))
//! );
//!
//! make_fn!(full_url<StrIter, (Option<&str>, Option<&str>, Option<&str>)>,
//!     do_each!(
//!         protocol => proto,
//!         // If we match the protocol then we must have a domain.
//!         // This is an example of an unrecoverable parsing error so we
//!         // abort with the must! macro if it doesn't match.
//!         domain => must!(domain),
//!         path => optional!(path),
//!         (Some(protocol), Some(domain), path)
//!     )
//! );
//!
//! make_fn!(relative_url<StrIter, (Option<&str>, Option<&str>, Option<&str>)>,
//!     do_each!(
//!         _ => not!(either!(text_token!("//"), proto)),
//!         // we require a valid path for relative urls.
//!         path => path,
//!         (None, None, Some(path))
//!     )
//! );
//!
//! make_fn!(url<StrIter, (Option<&str>, Option<&str>, Option<&str>)>,
//!     either!(
//!         full_url,
//!         relative_url,
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
//!     if let Some(domain) = domain {
//!         assert_eq!(domain, "example.com");
//!     }
//!     assert!(path.is_some());
//!     if let Some(path) = path {
//!         assert_eq!(path, "/some/path");
//!     }
//! }
//!
//! let bad_input = StrIter::new("http:///some/path");
//! let bad_result = url(bad_input);
//! assert!(bad_result.is_abort());
//! # }
//! ```
use std::fmt::{Debug, Display};
use std::iter::Iterator;
use std::result;

/// A trait for types that can have an offset as a count of processed items.
pub trait Offsetable {
    fn get_offset(&self) -> usize;
}

impl Offsetable for usize {
    fn get_offset(&self) -> usize {
        return *self;
    }
}

pub trait Seekable {
    fn seek(&mut self, u: usize) -> usize;
}

/// Trait for Inputs that can report current lines and columns in a text input.
pub trait Positioned {
    fn line(&self) -> usize;
    fn column(&self) -> usize;
}

/// SpanRange encompasses the valid Ops::Range types for use with the Span trait.
pub enum SpanRange {
    Range(std::ops::Range<usize>),
    RangeTo(std::ops::RangeTo<usize>),
    RangeFrom(std::ops::RangeFrom<usize>),
    RangeFull(std::ops::RangeFull),
}

/// An input that can provide a span of a range of the input.
pub trait Span<O> {
    fn span(&self, idx: SpanRange) -> O;
}

pub trait Peekable<O> {
    fn peek_next(&self) -> Option<O>;
}

/// A Cloneable Iterator that can report an offset as a count of processed Items.
pub trait InputIter: Iterator + Clone + Offsetable {
    fn curr(&self) -> Self::Item;
}

/// The custom error type for use in `Result::{Fail, Abort}`.
/// Stores a wrapped err that must implement Display as well as an offset and
/// an optional cause.
#[derive(Debug, Clone)]
pub struct Error<C> {
    msg: String,
    cause: Option<Box<Error<C>>>,
    context: Box<C>,
}

impl<C> Error<C> {
    /// Constructs a new Error with an offset and no cause.
    pub fn new<D: Into<String>>(msg: D, ctx: Box<C>) -> Self {
        Error {
            msg: msg.into(),
            cause: None,
            context: ctx,
        }
    }

    /// Constructs a new Error with an offset and a cause.
    pub fn caused_by<'a, D: Into<String>>(msg: D, cause: Box<Self>, ctx: Box<C>) -> Self {
        Error {
            msg: msg.into(),
            cause: Some(cause),
            context: ctx,
        }
    }

    /// Returns the msg.
    pub fn get_msg<'a>(&'a self) -> &str {
        &self.msg
    }

    /// Returns `Some(cause)` if there is one, None otherwise.
    pub fn get_cause<'a>(&'a self) -> Option<&'a Error<C>> {
        match self.cause {
            Some(ref e) => Some(e),
            None => None,
        }
    }

    pub fn get_context(&self) -> &C {
        self.context.as_ref()
    }
}

impl<C: Offsetable> Offsetable for Error<C> {
    // Returns the offset at which this Error happened.
    fn get_offset(&self) -> usize {
        self.context.get_offset()
    }
}

impl<C: Offsetable> Display for Error<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.msg)?;
        match self.cause {
            Some(ref c) => write!(f, "\n\tCaused By:{}", c),
            None => Ok(()),
        }
    }
}

impl<C: Offsetable + Debug> std::error::Error for Error<C> {}

/// The result of a parsing attempt.
#[derive(Debug)]
pub enum Result<I: InputIter, O> {
    /// Complete represents a successful match.
    Complete(I, O),
    /// Incomplete indicates input ended before a match could be completed.
    /// It contains the offset at which the input ended before a match could be completed.
    Incomplete(I),
    /// Fail represents a failed match.
    Fail(Error<I>),
    /// Abort represents a match failure that the parser cannot recover from.
    Abort(Error<I>),
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
pub use iter::StrIter;

#[macro_use]
pub mod combinators;
pub mod iter;

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod test;
