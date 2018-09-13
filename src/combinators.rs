//! Contains combinators that can assemble other matchers or combinators into more complex grammars.

use super::{InputIter, Error, Result};

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
            Result::Complete(i, _) => Result::Fail(Error::new("Matched on input when we shouldn't have.".to_string(), &i)),
            Result::Abort(e) => Result::Abort(e),
            Result::Incomplete(offset) => Result::Incomplete(offset),
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
            Result::Incomplete(offset) => Result::Incomplete(offset),
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
    I: InputIter
{
    match result {
        Result::Complete(i, o) => Result::Complete(i, o),
        Result::Incomplete(offset) => Result::Incomplete(offset),
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
            $crate::Result::Incomplete(offset) => $crate::Result::Incomplete(offset),
            $crate::Result::Fail(e) => $crate::Result::Fail($crate::Error::caused_by($e, &_i, e)),
            $crate::Result::Abort(e) => $crate::Result::Abort($crate::Error::caused_by($e, &_i, e)),
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
    I: InputIter
{
        match result {
            Result::Complete(i, o) => Result::Complete(i, o),
            Result::Incomplete(offset) => Result::Incomplete(offset),
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
pub fn must_complete<I, O, M>(result: Result<I, O>, msg: M) -> Result<I, O>
where
    I: InputIter,
    M: Into<String>,
{
    match result {
            Result::Complete(i, o) => Result::Complete(i, o),
            Result::Incomplete(ref offset) => Result::Abort(Error::new(msg, offset)),
            Result::Fail(e) => Result::Abort(e),
            Result::Abort(e) => Result::Abort(e),
        }
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
        $crate::combinators::must_complete($f!($i, $($args)*), $e)
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
            $crate::Result::Incomplete(offset) => {
                Result::Incomplete(offset)
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
            $crate::Result::Incomplete(offset) => {
                Result::Incomplete(offset)
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
            $crate::Result::Incomplete(i) => {
                Result::Incomplete(i)
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
            $crate::Result::Incomplete(i) => {
                Result::Incomplete(i)
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
            Result::Complete(i, o) => {
                Result::Complete(i, Some(o))
            }
            // Incomplete could still work possibly parse.
            Result::Incomplete(i) => {
                Result::Incomplete(i)
            }
            // Fail just means it didn't match.
            Result::Fail(_) => {
                Result::Complete(iter, None)
            },
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

/// Runs a single matcher repeating 0 or mre times and returns a possibly empty
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
        use $crate::{Result, Offsetable, Span, SpanRange};
        let start_offset = $i.get_offset();
        let mut _i = $i.clone();
        let pfn = || {
            loop {
                match $term!(_i.clone(), $($args)*) {
                    Result::Complete(_, _) => {
                        let range = SpanRange::Range(start_offset.._i.get_offset());
                        return Result::Complete(_i, $i.span(range));
                    },
                    Result::Abort(e) => return Result::Abort(e),
                    Result::Incomplete(offset) => return Result::Incomplete(offset),
                    Result::Fail(_) => {
                        // noop
                    }
                }
                if let None = _i.next() {
                    return Result::Incomplete(_i.get_offset());
                }
            }
        };
        pfn()
    }};

    ($i:expr, $term:ident) => {
        consume_until!($i, run!($term))
    };
}

/// Discards the output of a combinator rule when it completes and just returns `()`.
/// Leaves Failures, Aborts, and Incompletes untouched.
#[macro_export]
macro_rules! discard {
    ($i:expr, $term:ident) => {
        discard!($i, run!($term))
    };

    ($i:expr, $term:ident!( $( $args:tt )* ) ) => {{
        use $crate::Result;
        match $term!($i, $($args)*) {
            Result::Complete(i, _) => Result::Complete(i, ()),
            Result::Incomplete(offset) => Result::Incomplete(offset),
            Result::Fail(e) => Result::Fail(e),
            Result::Abort(e) => Result::Abort(e),
        }
    }};
}

/// Matches and returns any ascii charactar whitespace byte.
pub fn ascii_ws<'a, I: InputIter<Item=&'a u8>>(mut i: I) -> Result<I, u8> {
    match i.next() {
        Some(b) => {
            match b {
                b'\r' => Result::Complete(i, *b),
                b'\n' => Result::Complete(i, *b),
                b'\t' => Result::Complete(i, *b),
                b' ' => Result::Complete(i, *b),
                _ => Result::Fail(Error::new("Not whitespace", &i)),
            }
        },
        None => {
            Result::Fail(Error::new("Unexpected End Of Input", &i))
        }
    }
}

/// Matches the end of input for any InputIter.
/// Returns `()` for any match.
pub fn eoi<I: InputIter>(i: I) -> Result<I, ()> {
    let mut _i = i.clone();
    match _i.next() {
        Some(_) => Result::Fail(Error::new("Expected End Of Input", &i)),
        None => Result::Complete(i, ()),
    }
}

/// constructs a function named $name that takes an input of type $i and produces an output
/// of type $o.
///
#[macro_export]
macro_rules! make_fn {
    ($name:ident<$i:ty, $o:ty>, $rule:ident!($( $body:tt )* )) => {
        fn $name(i: $i) -> Result<$i,$o> {
            $rule!(i, $($body)*)
        }
    };
    
    (pub $name:ident<$i:ty, $o:ty>, $rule:ident!($( $body:tt )* )) => {
        pub fn $name(i: $i) -> Result<$i,$o> {
            $rule!(i, $($body)*)
        }
    };

    ($name:ident<$i:ty, $o:ty>, $rule:ident) => {
        make_fn!($name<$i, $o>, run!($rule))
    };

    (pub $name:ident<$i:ty, $o:ty>, $rule:ident) => {
        make_fn!(pub $name<$i, $o>, run!($rule))
    };

}