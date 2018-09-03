//! A parser combinator library with a focus on fully abortable parsing and error handling.
use std::iter::Iterator;

/// A Cloneable Iterator that can report an offset as a count of processed Items.
pub trait InputIter: Iterator + Clone {
    fn get_offset(&self) -> usize;
}

/// The result of a parsing attempt.
#[derive(Debug)]
pub enum Result<I: InputIter, O, E> {
    /// Complete represents a successful match.
    Complete(I, O),
    /// Incomplete indicates input ended before a match could be completed.
    /// It contains the offset at which the input ended before a match could be completed.
    Incomplete(usize),
    /// Fail represents a failed match.
    Fail(E),
    /// Abort represents a match failure that the parser cannot recover from.
    Abort(E),
}

impl<I: InputIter, O, E> Result<I, O, E> {
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