//! Contains implementations of `InputIter`.
use std::iter::Iterator;
use std::fmt::Debug;

use super::{Offsetable, InputIter};

/// Implements `InputIter` for any slice of T.
#[derive(Debug)]
pub struct SliceIter<'a, T: Debug + 'a> {
    source: &'a [T],
    offset: usize,
}

impl<'a, T: Debug + 'a> SliceIter<'a, T> {
    /// new constructs a SliceIter from a Slice of T.
    pub fn new(source: &'a [T]) -> Self {
        SliceIter {
            source: source,
            offset: 0,
        }
    }
}

impl<'a, T: Debug + 'a> Iterator for SliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.get(self.offset) {
            Some(item) => {
                self.offset += 1;
                Some(item)
            },
            None => None,
        }
    }
}

impl<'a, T: Debug + 'a> Offsetable for SliceIter<'a, T> {
    fn get_offset(&self) -> usize {
        self.offset
    }
}

impl<'a, T: Debug + 'a> Clone for SliceIter<'a, T> {
    fn clone(&self) -> Self {
        SliceIter {
            source: self.source,
            offset: self.offset,
        }
    }
}

impl<'a, T: Debug + 'a> InputIter for SliceIter<'a, T> {}