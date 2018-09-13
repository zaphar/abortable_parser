//! Contains implementations of `InputIter`.
use std::fmt::Debug;
use std::iter::Iterator;

use super::{InputIter, Offsetable, Span, SpanRange, TextPositionTracker};

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
            }
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

impl<'a, T: Debug + 'a> Span<&'a [T]> for SliceIter<'a, T> {
    fn span(&self, idx: SpanRange) -> &'a [T] {
        match idx {
            SpanRange::Range(r) => self.source.index(r),
            SpanRange::RangeTo(r) => self.source.index(r),
            SpanRange::RangeFrom(r) => self.source.index(r),
            SpanRange::RangeFull(r) => self.source.index(r),
        }
    }
}

impl<'a> From<&'a str> for SliceIter<'a, u8> {
    fn from(source: &'a str) -> Self {
        SliceIter::new(source.as_bytes())
    }
}

impl<'a, T: Debug> From<&'a [T]> for SliceIter<'a, T> {
    fn from(source: &'a [T]) -> Self {
        SliceIter::new(source)
    }
}

impl<'a, T: Debug> From<&'a Vec<T>> for SliceIter<'a, T> {
    fn from(source: &'a Vec<T>) -> Self {
        SliceIter::new(source.as_slice())
    }
}

/// Implements `InputIter` for any slice of T.
#[derive(Debug)]
pub struct StrIter<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
    column: usize,
}

impl<'a> StrIter<'a> {
    /// new constructs a StrIter from a Slice of T.
    pub fn new(source: &'a str) -> Self {
        StrIter {
            source: source,
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

impl<'a> Iterator for StrIter<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.as_bytes().get(self.offset) {
            // TODO count lines and columns.
            Some(item) => {
                self.offset += 1;
                if *item == b'\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                Some(item)
            }
            None => None,
        }
    }
}

impl<'a> Offsetable for StrIter<'a> {
    fn get_offset(&self) -> usize {
        self.offset
    }
}

impl<'a> TextPositionTracker for StrIter<'a> {
    fn line(&self) -> usize {
        self.line
    }

    fn column(&self) -> usize {
        self.column
    }
}

impl<'a> Clone for StrIter<'a> {
    fn clone(&self) -> Self {
        StrIter {
            source: self.source,
            offset: self.offset,
            line: self.line,
            column: self.column,
        }
    }
}

impl<'a> InputIter for StrIter<'a> {}

impl<'a> From<&'a str> for StrIter<'a> {
    fn from(source: &'a str) -> Self {
        Self::new(source)
    }
}

use std::ops::Index;

impl<'a> Span<&'a str> for StrIter<'a> {
    fn span(&self, idx: SpanRange) -> &'a str {
        match idx {
            SpanRange::Range(r) => self.source.index(r),
            SpanRange::RangeTo(r) => self.source.index(r),
            SpanRange::RangeFrom(r) => self.source.index(r),
            SpanRange::RangeFull(r) => self.source.index(r),
        }
    }
}
