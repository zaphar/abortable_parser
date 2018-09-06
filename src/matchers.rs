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
