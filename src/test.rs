use std::fmt::{Debug, Display};

use super::{InputIter, Offsetable, Result};
use combinators::*;
use iter::{SliceIter, StrIter};

#[test]
fn test_slice_iter() {
    let input_str = "foo";
    let mut iter = SliceIter::new(input_str.as_bytes());
    let cloned = iter.clone();
    assert_eq!(0, iter.get_offset());
    let mut out = Vec::new();
    loop {
        let b = match iter.next() {
            None => break,
            Some(b) => b,
        };
        out.push(b.clone());
    }
    assert_eq!(3, out.len());
    assert_eq!('f' as u8, out[0]);
    assert_eq!('o' as u8, out[1]);
    assert_eq!('o' as u8, out[2]);
    assert_eq!(3, iter.get_offset());

    out = Vec::new();
    for b in cloned {
        out.push(b.clone());
    }
    assert_eq!(3, out.len());
    assert_eq!('f' as u8, out[0]);
    assert_eq!('o' as u8, out[1]);
    assert_eq!('o' as u8, out[2]);
}

fn will_fail<I, C>(i: I) -> Result<I, String>
where
    I: InputIter<Item = C>,
    C: Debug + Display,
{
    Result::Fail(super::Error::new("AAAAHHH!!!".to_string(), &i))
}

fn parse_byte<'a, I>(mut i: I) -> Result<I, u8>
where
    I: InputIter<Item = &'a u8>,
{
    match i.next() {
        Some(b) => Result::Complete(i, *b),
        None => Result::Incomplete(i.get_offset()),
    }
}

fn will_not_complete<'a, I>(_: I) -> Result<I, String>
where
    I: InputIter<Item = &'a u8>,
{
    Result::Incomplete(0)
}

fn parse_three<'a, I>(i: I) -> Result<I, String>
where
    I: InputIter<Item = &'a u8>,
{
    let mut _i = i.clone();
    let mut out = String::new();
    loop {
        let b = match _i.next() {
            None => break,
            Some(b) => *b,
        };
        out.push(b as char);
        if out.len() == 3 {
            break;
        }
    }
    if out.len() != 3 {
        Result::Incomplete(_i.get_offset())
    } else {
        Result::Complete(_i, out)
    }
}

#[test]
fn test_peek() {
    let input_str = "foo bar";
    let iter = SliceIter::new(input_str.as_bytes());
    let pristine = iter.clone();
    let result = peek!(iter, text_token!("foo"));
    assert!(result.is_complete());
    if let Result::Complete(i, o) = result {
        assert_eq!(pristine.get_offset(), i.get_offset());
        assert_eq!("foo", o);
    }
}

#[test]
fn test_not_success() {
    let input_str = "foo bar";
    let iter = SliceIter::new(input_str.as_bytes());
    let pristine = iter.clone();
    let result = not!(iter, will_fail);
    assert!(result.is_complete());
    if let Result::Complete(i, o) = result {
        assert_eq!(pristine.get_offset(), i.get_offset());
        assert_eq!((), o);
    }
}

#[test]
fn test_not_fail() {
    let input_str = "foo bar";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = not!(iter, text_token!("foo"));
    assert!(result.is_fail());
}

#[test]
fn test_text_token() {
    let input_str = "foo bar";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = text_token!(iter, "foo");
    assert!(result.is_complete());
    if let Result::Complete(i, o) = result {
        assert_eq!(i.get_offset(), 3);
        assert_eq!(o, "foo");
    }
}

#[test]
fn test_text_token_fails() {
    let input_str = "foo bar";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = text_token!(iter, "bar");
    assert!(result.is_fail());
}

#[test]
fn test_wrap_err_fail() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = wrap_err!(iter, will_fail, "haha!".to_string());
    assert!(result.is_fail());
    if let Result::Fail(e) = result {
        assert!(e.get_cause().is_some());
        assert_eq!("AAAAHHH!!!", e.get_cause().unwrap().get_msg());
    }
}

#[test]
fn test_wrap_err_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = wrap_err!(iter, must!(will_fail), "haha!".to_string());
    assert!(result.is_abort());
    if let Result::Abort(e) = result {
        assert!(e.get_cause().is_some());
        assert_eq!("AAAAHHH!!!", e.get_cause().unwrap().get_msg());
    }
}

#[test]
fn test_must_fails() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = must!(iter, will_fail);
    assert!(result.is_abort());
}

#[test]
fn test_must_succeed() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = must!(iter, parse_byte);
    assert!(result.is_complete());
}

#[test]
fn test_trap_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = trap!(iter, must!(will_fail));
    assert!(result.is_fail(), format!("{:?}", result));
}

#[test]
fn test_trap_incomplete() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = trap!(iter, will_not_complete);
    assert!(result.is_incomplete(), format!("{:?}", result));
}

#[test]
fn test_trap_fail() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = trap!(iter, will_fail);
    assert!(result.is_fail(), format!("{:?}", result));
}

#[test]
fn test_trap_complete() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = trap!(iter, parse_byte);
    assert!(result.is_complete(), format!("{:?}", result));
}

#[test]
fn test_must_complete() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let iter_fail = iter.clone();
    let mut result = must_complete!(iter, "AHHH".to_string(), will_not_complete);
    assert!(result.is_abort());
    result = must_complete!(iter_fail, "AHHH".to_string(), will_fail);
    assert!(result.is_abort());
}

#[test]
fn test_do_each() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = do_each!(iter,
        b1 => parse_byte,
        _ => parse_byte,
        b3 => parse_byte,
        (b1, b3)
    );
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!('f' as u8, o.0);
        assert_eq!('o' as u8, o.1);
    } else {
        assert!(false, "did not get a tuple of 2 items");
    }
}

#[test]
fn test_either_idents() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = either!(iter, will_fail, will_fail, parse_three);
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!("foo".to_string(), o);
    } else {
        assert!(false, "Didn't not successfully match");
    }
}

#[test]
fn test_either_macros() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = either!(iter, run!(will_fail), run!(will_fail), run!(parse_three));
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!("foo".to_string(), o);
    } else {
        assert!(false, "Didn't successfully match");
    }
}

#[test]
fn test_either_fail() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = either!(iter, run!(will_fail), run!(will_fail));
    assert!(result.is_fail());
}

#[test]
fn test_either_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = either!(iter, must!(will_fail), parse_three, run!(will_fail));
    assert!(result.is_abort());
}

#[test]
fn test_optional_some() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = optional!(iter, parse_byte);
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!('f' as u8, o.unwrap());
    } else {
        assert!(false, "optional! did not complete");
    }
}

#[test]
fn test_optional_none() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = optional!(iter, will_fail);
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert!(o.is_none(), "output was not none");
    } else {
        assert!(false, "optional! did not complete");
    }
}

#[test]
fn test_optional_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = optional!(iter, must!(will_fail));
    assert!(result.is_abort(), "optional did not abort");
}

#[test]
fn test_repeat() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = repeat!(iter, parse_byte);
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!(3, o.len());
        assert_eq!('f' as u8, o[0]);
        assert_eq!('o' as u8, o[1]);
        assert_eq!('o' as u8, o[2]);
    } else {
        assert!(false, "repeat did not parse succesfully");
    }
}

#[test]
fn test_repeat_fail() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = repeat!(iter, will_fail);
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!(0, o.len());
    } else {
        assert!(false, "repeat did not parse succesfully");
    }
}

#[test]
fn test_repeat_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = repeat!(iter, must!(will_fail));
    assert!(result.is_abort());
}

#[test]
fn test_until() {
    let input_str = "foo; ";
    let iter = StrIter::new(input_str);
    let result = until!(iter, text_token!("; "));
    assert!(result.is_complete());
    if let Result::Complete(i, o) = result {
        assert_eq!(i.get_offset(), 3);
        assert_eq!(o.len(), 3);
        assert_eq!(o, "foo");
    }
}

#[test]
fn test_until_abort() {
    let input_str = "foo ";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = until!(iter, must!(will_fail));
    assert!(result.is_abort());
}

#[test]
fn test_until_incomplete() {
    let input_str = "foo;";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = until!(iter, text_token!("; "));
    assert!(result.is_incomplete());
}

#[test]
fn test_discard_success() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = discard!(iter, text_token!("foo"));
    assert!(result.is_complete());
    if let Result::Complete(_, o) = result {
        assert_eq!(o, ());
    }
}

#[test]
fn test_discard_fail() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = discard!(iter, will_fail);
    assert!(result.is_fail());
}

#[test]
fn test_discard_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = discard!(iter, must!(will_fail));
    assert!(result.is_abort());
}

#[test]
fn test_discard_incomplete() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = discard!(iter, will_not_complete);
    assert!(result.is_incomplete());
}

#[test]
fn test_eoi_success() {
    let input_str = "";
    let iter = StrIter::new(input_str);
    let result = eoi(iter);
    assert!(result.is_complete());
}

#[test]
fn test_eoi_fail() {
    let input_str = " ";
    let iter = StrIter::new(input_str);
    let result = eoi(iter);
    assert!(result.is_fail());
}

#[test]
fn test_ascii_ws_space() {
    let input_str = " ";
    let iter = StrIter::new(input_str);
    let result = ascii_ws(iter);
    assert!(result.is_complete());
}

#[test]
fn test_ascii_ws_tab() {
    let input_str = "\t";
    let iter = StrIter::new(input_str);
    let result = ascii_ws(iter);
    assert!(result.is_complete());
}

#[test]
fn test_ascii_ws_newline() {
    let input_str = "\n";
    let iter = StrIter::new(input_str);
    let result = ascii_ws(iter);
    assert!(result.is_complete());
}

#[test]
fn test_ascii_ws_carriage_return() {
    let input_str = "\r";
    let iter = StrIter::new(input_str);
    let result = ascii_ws(iter);
    assert!(result.is_complete());
}
