use super::iter::SliceIter;
use super::{Result, InputIter};

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

fn will_fail(_: SliceIter<u8>) -> Result<SliceIter<u8>, String, String>  {
    Result::Fail("AAAAHHH!!!".to_string())
}

fn parse_byte(mut i: SliceIter<u8>) -> Result<SliceIter<u8>, u8, String> {
    match i.next() {
        Some(b) => Result::Complete(i, *b),
        None => Result::Incomplete(i.get_offset()),
    }
}

fn will_not_complete(_: SliceIter<u8>) -> Result<SliceIter<u8>, String, String> {
    Result::Incomplete(0)
}

fn parse_three(i: SliceIter<u8>) -> Result<SliceIter<u8>, String, String> {
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
    let mut result = must_complete!(iter, |_| "AHHH".to_string(), will_not_complete);
    assert!(result.is_abort());
    result = must_complete!(iter_fail, |_| "AHHH".to_string(), will_fail);
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
    let result = either!(iter,
        will_fail,
        will_fail,
        parse_three);
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
    let result = either!(iter,
        run!(will_fail),
        run!(will_fail),
        run!(parse_three));
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
    let result = either!(iter,
        run!(will_fail),
        run!(will_fail));
    assert!(result.is_fail());
}

#[test]
fn test_either_abort() {
    let input_str = "foo";
    let iter = SliceIter::new(input_str.as_bytes());
    let result = either!(iter,
        must!(will_fail),
        parse_three,
        run!(will_fail));
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