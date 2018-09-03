//! A parser combinator library with a focus on fully abortable parsing and error handling.
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

/// A Cloneable Iterator that can report an offset as a count of processed Items.
pub trait InputIter: Iterator + Clone + Offsetable {}

/// The custom error type for use in `Result::{Fail, Abort}`.
/// Stores a wrapped err that must implement Display as well as an offset and
/// an optional cause.
#[derive(Debug)]
pub struct Error<E: Display> {
    err: E,
    offset: usize,
    cause: Option<Box<Error<E>>>,
}

impl<E: Display> Error<E> {
    /// Constructs a new Error with an offset and no cause.
    pub fn new<S: Offsetable>(err: E, offset: &S) -> Self {
        Error {
            err: err,
            offset: offset.get_offset(),
            cause: None,
        }
    }

    /// Constructs a new Error with an offset and a cause.
    pub fn caused_by<S: Offsetable>(err: E, offset: &S, cause: Self) -> Self {
        Error {
            err: err,
            offset: offset.get_offset(),
            cause: Some(Box::new(cause)),
        }
    }

    /// Returns the contained err.
    pub fn get_err<'a>(&'a self) -> &'a E {
        &self.err
    }

    /// Returns `Some(cause)` if there is one, None otherwise.
    pub fn get_cause<'a>(&'a self) -> Option<&'a Error<E>> {
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

impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        try!(write!(f, "{}", self.err));
        match self.cause {
            Some(ref c) => write!(f, "\n\tCaused By:{}", c),
            None => Ok(()),
        }
    }
}

/// The result of a parsing attempt.
#[derive(Debug)]
pub enum Result<I: InputIter, O, E: Display> {
    /// Complete represents a successful match.
    Complete(I, O),
    /// Incomplete indicates input ended before a match could be completed.
    /// It contains the offset at which the input ended before a match could be completed.
    Incomplete(usize),
    /// Fail represents a failed match.
    Fail(Error<E>),
    /// Abort represents a match failure that the parser cannot recover from.
    Abort(Error<E>),
}

impl<I: InputIter, O, E: Display> Result<I, O, E> {
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

pub use iter::SliceIter;

#[macro_use]
pub mod macros;
pub mod iter;

#[cfg(test)]
mod test;
