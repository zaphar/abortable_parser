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

//! Contains combinators that can assemble other matchers or combinators into more complex grammars.
use super::{Error, InputIter, Result};

/// Turns a `Result` to it's inverse.
///
/// `Result::Fail` becomes `Result::Complete` and `Result::Complete` becomes `Result::Fail`.
/// You must pass in an iterator at the appropriate spot for the next combinator
/// to start at.
///
/// The `not!` macro provides syntactic sugar for using this combinator properly.
pub fn not<I, O>(i: I, result: Result<I, O>) -> Result<I, ()>
where
    I: InputIter,
{
    match result {
        Result::Complete(i, _) => Result::Fail(Error::new(
            "Matched on input when we shouldn't have.",
            Box::new(i.clone()),
        )),
        Result::Abort(e) => Result::Abort(e),
        Result::Incomplete(ctx) => Result::Incomplete(ctx),
        Result::Fail(_) => Result::Complete(i, ()),
    }
}

/// Turns a matcher into it's inverse, only succeeding if the the matcher returns a Fail.
/// Does not consume it's input and only returns ().
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// # use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # use std::convert::From;
/// # fn main() {
/// # let iter: iter::SliceIter<u8> = "foo".into();
/// let tok = not!(iter, text_token!("bar"));
/// assert!(tok.is_complete());
/// if let Result::Complete(i, o) = tok {
///     assert_eq!(i.get_offset(), 0);
///     assert_eq!(o, ());
/// }
/// # }
/// ```
#[macro_export]
macro_rules! not {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {{
        let _i = $i.clone();
        $crate::combinators::not(_i, trap!($i.clone(), $f!($($args)*)))
    }};

    ($i:expr, $f:ident( $( $args:tt )* ) ) => {
        not!($i, run!($f($($args)*)))
    };

    ($i:expr, $f:ident) => {
        not!($i, run!($f))
    };
}

/// Checks the given matcher without consuming the input.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// # use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # use std::convert::From;
/// # fn main() {
/// # let iter: iter::SliceIter<u8> = "foo".into();
/// let tok = peek!(iter, text_token!("foo"));
/// # assert!(tok.is_complete());
/// # if let Result::Complete(i, o) = tok {
/// #     assert_eq!(i.get_offset(), 0);
/// #     assert_eq!(o, "foo");
/// # }
/// # }
/// ```
#[macro_export]
macro_rules! peek {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {{
        use $crate::Result;
        let _i = $i.clone();
        match $f!(_i, $($args)*) {
            Result::Complete(_, o) => Result::Complete($i, o),
            Result::Incomplete(ctx) => Result::Incomplete(ctx),
            Result::Abort(e) => Result::Abort(e),
            Result::Fail(e) => Result::Fail(e),
        }
    }};

    ($i:expr, $f:ident( $( $args:tt )* ) ) => {
        peek!($i, run!($f($($args)*)))
    };

    ($i:expr, $f:ident) => {
        peek!($i, run!($f))
    };
}

/// Converts a function indentifier into a macro call. Useful when writing your own macro combinator.
#[macro_export]
macro_rules! run {
    ($i:expr, $f:ident) => {
        $f($i)
    };
}

/// Maps a `Result::Fail` to a `Result::Abort`.
///
/// It leaves the rest of the Result variants untouched.
///
/// The `must!` macro provided syntactice sugar for using this combinator.
pub fn must<I, O>(result: Result<I, O>) -> Result<I, O>
where
    I: InputIter,
{
    match result {
        Result::Complete(i, o) => Result::Complete(i, o),
        Result::Incomplete(ctx) => Result::Incomplete(ctx),
        Result::Fail(e) => Result::Abort(e),
        Result::Abort(e) => Result::Abort(e),
    }
}

/// Turns `Result::Fail` into `Result::Abort`.
///
/// Allows you to turn any parse failure into a hard abort of the parser.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::Result;
/// # use std::convert::From;
/// # fn main() {
///
/// let iter: iter::SliceIter<u8> = "foo".into();
///
/// let tok = must!(iter, text_token!("foo"));
/// # assert!(tok.is_complete());
///
/// let fail = must!(iter, text_token!("bar"));
/// # assert!(fail.is_abort());
/// # }
/// ```
#[macro_export]
macro_rules! must {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {
        $crate::combinators::must($f!($i, $($args)*))
    };

    ($i:expr, $f:ident) => {
        must!($i, run!($f))
    };
}

/// Wraps any Error return from a subparser in another error. Stores the position at
/// this point in the parse tree allowing you to associate context with wrapped errors.
#[macro_export]
macro_rules! wrap_err {
    ($i:expr, $f:ident!( $( $args:tt )* ), $e:expr) => {{
        let _i = $i.clone();
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, o) => $crate::Result::Complete(i, o),
            $crate::Result::Incomplete(ctx) => $crate::Result::Incomplete(ctx),
            $crate::Result::Fail(e) => $crate::Result::Fail($crate::Error::caused_by($e, Box::new(e), Box::new(_i.clone()))),
            $crate::Result::Abort(e) => $crate::Result::Abort($crate::Error::caused_by($e, Box::new(e), Box::new(_i.clone()))),
        }
    }};

    ($i:expr, $f:ident( $( $args:tt )* ), $e:expr ) => {
        wrap_err!($i, run!($f($($args)*)), $e:expr)
    };

    ($i:expr, $f:ident, $e:expr) => {
        wrap_err!($i, run!($f), $e)
    };
}

/// Traps a `Result::Abort` and converts it into a `Result::Fail`.
///
/// This is the semantic inverse of `must`.
///
/// The `trap!` macro provides syntactic sugar for using this combinator.
pub fn trap<I, O>(result: Result<I, O>) -> Result<I, O>
where
    I: InputIter,
{
    match result {
        Result::Complete(i, o) => Result::Complete(i, o),
        Result::Incomplete(ctx) => Result::Incomplete(ctx),
        Result::Fail(e) => Result::Fail(e),
        Result::Abort(e) => Result::Fail(e),
    }
}

/// Turns `Result::Abort` into `Result::Fail` allowing you to trap and then convert any `Result::Abort`
/// into a normal Fail.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "foo";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = trap!(iter, must!(text_token!("bar")));
/// # assert!(result.is_fail());
/// # }
/// ```
#[macro_export]
macro_rules! trap {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {
        $crate::combinators::trap($f!($i, $($args)*))
    };

    ($i:expr, $f:ident) => {
        trap!($i, run!($f))
    };
}

/// Turns `Result::Fail` or `Result::Incomplete` into `Result::Abort`.
///
/// You must specify the error message to use in case the matcher is incomplete.
///
/// The must_complete! macro provides syntactic sugar for using this combinator.
pub fn must_complete<I, O>(result: Result<I, O>, msg: String) -> Result<I, O>
where
    I: InputIter,
{
    match result {
        Result::Complete(i, o) => Result::Complete(i, o),
        Result::Incomplete(ctx) => Result::Abort(Error::new(msg, Box::new(ctx))),
        Result::Fail(e) => Result::Abort(e),
        Result::Abort(e) => Result::Abort(e),
    }
}

/// Turns `Result::Incomplete` into `Result::Fail`.
pub fn complete<I, O, S>(result: Result<I, O>, msg: S) -> Result<I, O>
where
    I: InputIter,
    S: Into<String>,
{
    match result {
        Result::Incomplete(ctx) => Result::Fail(Error::new(msg.into(), Box::new(ctx))),
        Result::Complete(i, o) => Result::Complete(i, o),
        Result::Fail(e) => Result::Fail(e),
        Result::Abort(e) => Result::Abort(e),
    }
}

/// Turns  `Result::Incomplete` into `Result::Fail`.
#[macro_export]
macro_rules! complete {
    ($i:expr, $e:expr, $f:ident!( $( $args:tt )* ) ) => {
        $crate::combinators::complete($f!($i, $($args)*), $e)
    };

    ($i:expr, $efn:expr, $f:ident) => {
        complete!($i, $efn, run!($f))
    };
}

/// Turns `Result::Fail` and `Result::Incomplete` into `Result::Abort`.
///
/// You must specify the error message to use in case the matcher is incomplete.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "foo";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let mut result = must_complete!(iter, "AHHH".to_string(), text_token!("fooooo"));
/// # assert!(result.is_abort());
/// # }
#[macro_export]
macro_rules! must_complete {
    ($i:expr, $e:expr, $f:ident!( $( $args:tt )* ) ) => {{
        $crate::combinators::must_complete($f!($i.clone(), $($args)*), $e)
    }};

    ($i:expr, $efn:expr, $f:ident) => {
        must_complete!($i, $efn, run!($f))
    };
}

/// Captures a sequence of sub parsers output.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "(foobar)";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = do_each!(iter,
///     _ => text_token!("("),
///     foo => text_token!("foo"),
///     bar => text_token!("bar"),
///     _ => text_token!(")"),
///     (foo, bar) // This expression will be the result of the parse
/// );
/// # assert!(result.is_complete());
/// if let Result::Complete(_, o) = result {
///     assert_eq!("foo", o.0);
///     assert_eq!("bar", o.1);
/// }
/// # }
/// ```
///  
/// Or alternatively rather than a tuple as the output you can return a single
/// expression.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// # use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// # let input_str = "(foobar)";
/// # let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = do_each!(iter,
///     _ => text_token!("("),
///     foo => text_token!("foo"),
///     bar => text_token!("bar"),
///     _ => text_token!(")"),
///     (vec![foo, bar]) // Non tuple expression as a result.
/// );
/// # assert!(result.is_complete());
/// if let Result::Complete(_, o) = result {
///     assert_eq!(vec!["foo", "bar"], o);
/// }
/// # }
/// ```
///
/// The output from this combinator must be indicated by parentheses.
#[macro_export]
macro_rules! do_each {
    ($i:expr, $val:ident => $f:ident) => {
        // This is a compile failure.
        compile_error!("do_each! must end with a tuple capturing the results")
    };

    ($i:expr, $val:ident => $f:ident!($( $args:tt )* ), $($rest:tt)* ) => {
        // If any single one of these matchers fails then all of them are failures.
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, o) => {
                let $val = o;
                do_each!(i, $($rest)*)
            }
            $crate::Result::Incomplete(ctx) => {
                Result::Incomplete(ctx)
            }
            $crate::Result::Fail(e) => Result::Fail(e),
            $crate::Result::Abort(e) => Result::Abort(e),
        }
    };

    ($i:expr, _ => $f:ident!($( $args:tt )* ), $($rest:tt)* ) => {
        // If any single one of these matchers fails then all of them are failures.
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, _) => {
                do_each!(i, $($rest)*)
            }
            $crate::Result::Incomplete(ctx) => {
                Result::Incomplete(ctx)
            }
            $crate::Result::Fail(e) => Result::Fail(e),
            $crate::Result::Abort(e) => Result::Abort(e),
        }
    };

    ($i:expr, $val:ident => $f:ident, $($rest:tt)* ) => {
        // If any single one of these matchers fails then all of them are failures.
        do_each!($i, $val => run!($f), $( $rest )* )
    };

    ($i:expr, _ => $f:ident, $($rest:tt)* ) => {
        // If any single one of these matchers fails then all of them are failures.
        do_each!($i, _ => run!($f), $( $rest )* )
    };

    // Our Terminal condition
    ($i:expr, ( $($rest:tt)* ) ) => {
        Result::Complete($i, ($($rest)*))
    };
}

/// Returns the output of the first sub parser to succeed.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "foo";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = either!(iter, text_token!("bar"), text_token!("foo"));
/// # assert!(result.is_complete());
/// # if let Result::Complete(_, o) = result {
/// #     assert_eq!("foo", o);
/// # } else {
/// #     assert!(false, "either! did not complete");
/// # }
/// # }
#[macro_export]
macro_rules! either {
    // Initialization case.
    ($i:expr, $f:ident!( $( $args:tt )* ), $( $rest:tt)* ) => { // 0
        either!(__impl $i, $f!( $($args)* ), $($rest)*)
    };

    // Initialization case.
    ($i:expr, $f:ident, $($rest:tt)* ) => { // 1
        either!(__impl $i, run!($f), $($rest)*)
    };

    // Initialization failure case.
    ($i:expr, $f:ident!( $( $args:tt )* )) => { // 2
        compile_error!("Either requires at least two sub matchers.")
    };

    // Initialization failure case.
    ($i:expr, $f:ident) => { // 3
        either!($i, run!($f))
    };

    // Termination clause
    (__impl $i:expr, $f:ident) => { // 4
        either!(__impl $i, run!($f))
    };

    // Termination clause
    (__impl $i:expr, $f:ident,) => { // 5
        either!(__impl $i, run!($f))
    };

    // Termination clause
    (__impl $i:expr, $f:ident!( $( $args:tt )* ),) => { // 6
        either!(__impl $i, $f!($($args)*) __end)
    };

    // Termination clause
    (__impl $i:expr, $f:ident!( $( $args:tt )* )) => {{ // 7
        match $f!($i, $($args)*) {
            // The first one to match is our result.
            $crate::Result::Complete(i, o) => {
                Result::Complete(i, o)
            }
            // Incompletes may still be parseable.
            $crate::Result::Incomplete(ctx) => {
                Result::Incomplete(ctx)
            }
            // Fail means it didn't match so we are now done.
            $crate::Result::Fail(e) => {
                $crate::Result::Fail(e)
            },
            // Aborts are hard failures that the parser can't recover from.
            $crate::Result::Abort(e) => Result::Abort(e),
        }
    }};

    // Internal Loop Implementation
    (__impl $i:expr, $f:ident!( $( $args:tt )* ), $( $rest:tt )* ) => {{ // 8
        let _i = $i.clone();
        match $f!($i, $($args)*) {
            // The first one to match is our result.
            $crate::Result::Complete(i, o) => {
                Result::Complete(i, o)
            }
            // Incompletes may still be parseable.
            $crate::Result::Incomplete(ctx) => {
                Result::Incomplete(ctx)
            }
            // Fail means it didn't match so continue to next one.
            $crate::Result::Fail(_) => {
                either!(__impl _i, $($rest)*)
            },
            // Aborts are hard failures that the parser can't recover from.
            $crate::Result::Abort(e) => Result::Abort(e),
        }
    }};

    // Internal Loop Implementation
    (__impl $i:expr, $f:ident, $( $rest:tt )* ) => { // 9
        either!(__impl $i, run!($f), $( $rest )* )
    }
}

/// Maps a `Result` to be optional.
///
/// `Result::Fail` maps to None and `Result::Complete` maps to Some. The rest of the
/// `Result` variants are left untouched. You must pass in the iterator that the
/// next matcher should use in the event of a fail.
///
/// The `optional!` macro provides some syntactice sugar for using this combinator
/// properly.
pub fn optional<I, O>(iter: I, result: Result<I, O>) -> Result<I, Option<O>>
where
    I: InputIter,
{
    match result {
        Result::Complete(i, o) => Result::Complete(i, Some(o)),
        // Incomplete could still work possibly parse.
        Result::Incomplete(ctx) => Result::Incomplete(ctx),
        // Fail just means it didn't match.
        Result::Fail(_) => Result::Complete(iter, None),
        // Aborts are hard failures that the parser can't recover from.
        Result::Abort(e) => Result::Abort(e),
    }
}

/// Treats a sub parser as optional. It returns Some(output) for a successful match
/// and None for failures.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "foo";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = optional!(iter, text_token!("foo"));
/// # assert!(result.is_complete());
/// # if let Result::Complete(_, o) = result {
/// #     assert_eq!("foo", o.unwrap());
/// # } else {
/// #     assert!(false, "optional! did not complete");
/// # }
/// # }
#[macro_export]
macro_rules! optional {
    ($i:expr, $f:ident) => {
        optional!(__impl $i, run!($f))
    };

    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {
        optional!(__impl $i, $f!( $( $args )* ))
    };

    (__impl $i:expr, $f:ident!( $( $args:tt )* )) => {{
        let _i = $i.clone();
       $crate::combinators::optional(_i, $f!($i, $($args)*))
    }};
}

/// Runs a single matcher repeating 0 or more times and returns a possibly empty
/// vector of the parsed results.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "foofoo";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = repeat!(iter, text_token!("foo"));
/// # assert!(result.is_complete());
/// if let Result::Complete(_, o) = result {
///     assert_eq!(2, o.len());
///     assert_eq!("foo", o[0]);
///     assert_eq!("foo", o[1]);
/// }
/// # }
/// ```
#[macro_export]
macro_rules! repeat {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {{
        let mut _i = $i.clone();
        let mut seq = Vec::new();
        let mut opt_error = None;
        loop {
            let __i = _i.clone();
            match $f!(_i, $($args)*) {
                $crate::Result::Complete(i, o) => {
                    seq.push(o);
                    _i = i;
                }
                // Aborts are always a hard fail.
                $crate::Result::Abort(e) => {
                    opt_error = Some($crate::Result::Abort(e));
                    _i = $i.clone();
                    break;
                }
                // Everything else just means we are finished parsing.
                $crate::Result::Incomplete(_) => {
                    _i = __i;
                    break;
                }
                $crate::Result::Fail(_) => {
                    _i = __i;
                    break;
                }
            }
        }
        match opt_error {
            Some(e) => e,
            None => $crate::Result::Complete(_i, seq),
        }
    }};

    ($i:expr, $f:ident) => {
        repeat!($i, run!($f))
    };
}

/// Parses separated list of items.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # fn main() {
/// let input_str = "foo,foo";
/// let iter = iter::SliceIter::new(input_str.as_bytes());
/// let result = separated!(iter, text_token!(","), text_token!("foo"));
/// # assert!(result.is_complete());
/// if let Result::Complete(_, o) = result {
///     assert_eq!(2, o.len());
///     assert_eq!("foo", o[0]);
///     assert_eq!("foo", o[1]);
/// }
/// # }
/// ```
#[macro_export]
macro_rules! separated {
    ($i:expr, $sep_rule:ident!( $( $sep_args:tt )* ), $item_rule:ident!( $( $item_args:tt )* ) ) => {{
        use $crate::Result;
        let _i = $i.clone();
        // We require at least one item for our list
        let head =  $item_rule!($i.clone(), $($item_args)*);
        match head {
            Result::Incomplete(ctx) => Result::Incomplete(ctx),
            Result::Fail(e) => Result::Fail(e),
            Result::Abort(e) => Result::Abort(e),
            Result::Complete(i,item) => {
                let mut list = vec![item];
                // Now we parse a repeat of sep_rule and item_rule.
                let tail_result = repeat!(i,
                    do_each!(
                        _    => $sep_rule!($($sep_args)*),
                        item => $item_rule!($($item_args)*),
                        (item)
                    )
                );
                match tail_result {
                    Result::Fail(e) => Result::Fail(e),
                    Result::Incomplete(ctx) => Result::Incomplete(ctx),
                    Result::Abort(e) => Result::Abort(e),
                    Result::Complete(i, mut tail) => {
                        list.extend(tail.drain(0..));
                        Result::Complete(i, list)
                    }
                }
            }
        }
    }};

    ($i:expr, $sep_rule:ident, $item_rule:ident ) => {
        separated!($i, run!($sep_rule), run!($item_rule))
    };

    ($i:expr, $sep_rule:ident!( $( $args:tt )* ), $item_rule:ident ) => {
        separated!($i, $sep_rule!($($args)*), run!($item_rule))
    };

    ($i:expr, $sep_rule:ident, $item_rule:ident!( $( $args:tt )* ) ) => {
        separated!($i, run!($sep_rule), $item_rule!($($args)*))
    };
}

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
                Box::new($i.clone()),
            ))
        }
    }};
}

/// Consumes an input until it reaches a term that the contained rule matches.
/// It does not consume the subrule.
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
    ($i:expr, $rule:ident!( $( $args:tt )* ) ) => {{
        use $crate::{Result, Offsetable, Span, SpanRange};
        let start_offset = $i.get_offset();
        let mut _i = $i.clone();
        let pfn = || {
            loop {
                match $rule!(_i.clone(), $($args)*) {
                    Result::Complete(_, _) => {
                        let range = SpanRange::Range(start_offset.._i.get_offset());
                        return Result::Complete(_i, $i.span(range));
                    },
                    Result::Abort(e) => return Result::Abort(e),
                    Result::Incomplete(ctx) => return Result::Incomplete(ctx),
                    Result::Fail(_) => {
                        // noop
                    }
                }
                if let None = _i.next() {
                    return Result::Incomplete(_i.clone());
                }
            }
        };
        pfn()
    }};

    ($i:expr, $rule:ident) => {
        until!($i, run!($rule))
    };
}

/// Discards the output of a combinator rule when it completes and just returns `()`.
/// Leaves Failures, Aborts, and Incompletes untouched.
#[macro_export]
macro_rules! discard {
    ($i:expr, $rule:ident) => {
        discard!($i, run!($rule))
    };

    ($i:expr, $rule:ident!( $( $args:tt )* ) ) => {{
        use $crate::Result;
        match $rule!($i, $($args)*) {
            Result::Complete(i, _) => Result::Complete(i, ()),
            Result::Incomplete(ctx) => Result::Incomplete(ctx),
            Result::Fail(e) => Result::Fail(e),
            Result::Abort(e) => Result::Abort(e),
        }
    }};
}

/// Matches and returns any ascii charactar whitespace byte.
pub fn ascii_ws<'a, I: InputIter<Item = &'a u8>>(mut i: I) -> Result<I, u8> {
    match i.next() {
        Some(b) => {
            if (*b as char).is_whitespace() {
                Result::Complete(i, *b)
            } else {
                Result::Fail(Error::new(
                    "Not whitespace".to_string(),
                    Box::new(i.clone()),
                ))
            }
        }
        None => Result::Fail(Error::new(
            "Unexpected End Of Input".to_string(),
            Box::new(i.clone()),
        )),
    }
}

/// Matches the end of input for any InputIter.
/// Returns `()` for any match.
pub fn eoi<I: InputIter>(i: I) -> Result<I, ()> {
    let mut _i = i.clone();
    match _i.next() {
        Some(_) => Result::Fail(Error::new(
            "Expected End Of Input".to_string(),
            Box::new(i.clone()),
        )),
        None => Result::Complete(i, ()),
    }
}

/// Constructs a function named $name that takes an input of type $i and produces an output
/// of type $o.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// # use abortable_parser::iter::StrIter;
/// make_fn!(myrule<StrIter, &str>,
///     text_token!("token")
/// );
/// ```
///
/// You can also specify that the function is public if so desired.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// # use abortable_parser::iter::StrIter;
/// make_fn!(pub otherrule<StrIter, &str>,
///     text_token!("other")
/// );
/// ```
#[macro_export]
macro_rules! make_fn {
    ($name:ident<$i:ty, $o:ty>, $rule:ident!($( $body:tt )* )) => {
        fn $name(i: $i) -> $crate::Result<$i, $o> {
            $rule!(i, $($body)*)
        }
    };

    (pub $name:ident<$i:ty, $o:ty>, $rule:ident!($( $body:tt )* )) => {
        pub fn $name(i: $i) -> $crate::Result<$i, $o> {
            $rule!(i, $($body)*)
        }
    };

    ($name:ident<$i:ty, $o:ty>, $rule:ident) => {
        make_fn!($name<$i, $o>, run!($rule));
    };

    (pub $name:ident<$i:ty, $o:ty>, $rule:ident) => {
        make_fn!(pub $name<$i, $o>, run!($rule));
    };
}

/// Helper macro that returns the input without consuming it.
///
/// Useful when you need to get the input and use it to retrieve
/// positional information like offset or line and column.
#[macro_export]
macro_rules! input {
    ($i:expr) => {
        input!($i,)
    };

    ($i:expr,) => {{
        let _i = $i.clone();
        $crate::Result::Complete($i, _i)
    }};
}

/// Consumes the input until the $rule fails and then returns the consumed input as
/// a slice.
///
/// ```
/// # #[macro_use] extern crate abortable_parser;
/// use abortable_parser::iter;
/// # use abortable_parser::{Result, Offsetable};
/// # use abortable_parser::combinators::ascii_alpha;
/// use std::convert::From;
/// # fn main() {
/// let iter: iter::StrIter = "foo;".into();
/// let tok = consume_all!(iter, ascii_alpha);
/// # assert!(tok.is_complete());
/// if let Result::Complete(i, o) = tok {
///     assert_eq!(i.get_offset(), 3);
///     assert_eq!(o, "foo");
/// }
/// # }
/// ```
#[macro_export]
macro_rules! consume_all {
    ($i:expr, $rule:ident!( $( $args:tt )* ) ) => {{
        use $crate::{Result, Offsetable, Span, SpanRange};
        let start_offset = $i.get_offset();
        let mut _i = $i.clone();
        let pfn = || {
            loop {
                match $rule!(_i.clone(), $($args)*) {
                    Result::Complete(_, _) => {
                        // noop
                    },
                    Result::Abort(e) => return Result::Abort(e),
                    Result::Incomplete(ctx) => return Result::Incomplete(ctx),
                    Result::Fail(_) => {
                        let range = SpanRange::Range(start_offset.._i.get_offset());
                        return Result::Complete(_i, $i.span(range));
                    }
                }
                if let None = _i.next() {
                    return Result::Incomplete(_i.clone());
                }
            }
        };
        pfn()
    }};

    ($i:expr, $rule:ident) => {
        consume_all!($i, run!($rule))
    }
}

/// ascii_digit parses a single ascii alphabetic or digit character from an InputIter of bytes.
#[inline(always)]
pub fn ascii_alphanumeric<'a, I: InputIter<Item = &'a u8>>(mut i: I) -> Result<I, u8> {
    match i.next() {
        Some(b) => {
            let c = *b as char;
            if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                Result::Complete(i, *b)
            } else {
                Result::Fail(Error::new(
                    "Not an alphanumeric character".to_string(),
                    Box::new(i.clone()),
                ))
            }
        }
        None => Result::Fail(Error::new(
            "Unexpected End Of Input.".to_string(),
            Box::new(i.clone()),
        )),
    }
}

/// ascii_digit parses a single ascii digit character from an InputIter of bytes.
#[inline(always)]
pub fn ascii_digit<'a, I: InputIter<Item = &'a u8>>(mut i: I) -> Result<I, u8> {
    match i.next() {
        Some(b) => {
            if (*b as char).is_ascii_digit() {
                Result::Complete(i, *b)
            } else {
                Result::Fail(Error::new(
                    "Not an digit character".to_string(),
                    Box::new(i.clone()),
                ))
            }
        }
        None => Result::Fail(Error::new(
            "Unexpected End Of Input.".to_string(),
            Box::new(i.clone()),
        )),
    }
}

/// ascii_alpha parses a single ascii alphabet character from an InputIter of bytes.
#[inline(always)]
pub fn ascii_alpha<'a, I: InputIter<Item = &'a u8>>(mut i: I) -> Result<I, u8> {
    match i.next() {
        Some(b) => {
            if (*b as char).is_ascii_alphabetic() {
                Result::Complete(i, *b)
            } else {
                Result::Fail(Error::new(
                    "Not an alpha character".to_string(),
                    Box::new(i.clone()),
                ))
            }
        }
        None => Result::Fail(Error::new(
            "Unexpected End Of Input.".to_string(),
            Box::new(i.clone()),
        )),
    }
}

// TODO(jwall): We need a helper to convert Optional into failures.
// TODO(jwall): We need a helper to convert std::result::Result into failures.
