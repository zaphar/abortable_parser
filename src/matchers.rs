//! Contains matchers for matching specific patterns or tokens.

/// Convenience macro for looking for a specific text token in a byte input stream.
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