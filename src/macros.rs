//! Contains the helper macros for abortable-parser.

/// Converts a function indentifier into a macro call. Useful when writing your own macro combinator.
#[macro_export]
macro_rules! run {
    ($i:expr, $f:ident) => {
        $f($i)
    };
}

/// Turns Fails into Aborts. Allows you to turn any parse failure into a hard abort of the parser.
#[macro_export]
macro_rules! must {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, o) => $crate::Result::Complete(i, o),
            $crate::Result::Incomplete(offset) => $crate::Result::Incomplete(offset),
            $crate::Result::Fail(e) => $crate::Result::Abort(e),
            $crate::Result::Abort(e) => $crate::Result::Abort(e),
        }
    };
    
    ($i:expr, $f:ident) => {
        must!($i, run!($f))
    };
}

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
    
    ($i:expr, $f:ident, $e:expr) => {
        wrap_err!($i, run!($f), $e)
    };
}

/// Turns Aborts into fails allowing you to trap and then convert an Abort into a normal Fail.
#[macro_export]
macro_rules! trap {
    ($i:expr, $f:ident!( $( $args:tt )* ) ) => {
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, o) => $crate::Result::Complete(i, o),
            $crate::Result::Incomplete(offset) => $crate::Result::Incomplete(offset),
            $crate::Result::Fail(e) => $crate::Result::Fail(e),
            $crate::Result::Abort(e) => $crate::Result::Fail(e),
        }
    };
    
    ($i:expr, $f:ident) => {
        trap!($i, run!($f))
    };
}

/// Turns Fails and Incompletes into Aborts. It uses an error factory
/// to construct the errors for the Incomplete case.
#[macro_export]
macro_rules! must_complete {
    ($i:expr, $e:expr, $f:ident!( $( $args:tt )* ) ) => {{
        let _i = $i.clone();
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, o) => $crate::Result::Complete(i, o),
            $crate::Result::Incomplete(ref offset) => $crate::Result::Abort($crate::Error::new($e, offset)),
            $crate::Result::Fail(e) => $crate::Result::Abort(e),
            $crate::Result::Abort(e) => $crate::Result::Abort(e),
        }
    }};
    
    ($i:expr, $efn:expr, $f:ident) => {
        must_complete!($i, $efn, run!($f))
    };
}

/// Captures a sequence of sub parsers output.
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

/// Treats a sub parser as optional. It returns Some(output) for a successful match
/// and None for Fails.
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
        match $f!($i, $($args)*) {
            $crate::Result::Complete(i, o) => {
                Result::Complete(i, Some(o))
            }
            // Incomplete could still work possibly parse.
            $crate::Result::Incomplete(i) => {
                Result::Incomplete(i)
            }
            // Fail just means it didn't match.
            $crate::Result::Fail(_) => {
                Result::Complete(_i, None)
            },
            // Aborts are hard failures that the parser can't recover from.
            $crate::Result::Abort(e) => Result::Abort(e),
        }
    }};
}

/// Runs a single parser repeating 0 or mre times and returns a possibly empty
/// vector of the parsed results.
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
