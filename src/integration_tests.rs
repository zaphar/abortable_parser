
use super::{ascii_ws, eoi, Result};
use iter::StrIter;

make_fn!(proto<StrIter, &str>,
     do_each!(
         proto => until!(text_token!("://")),
         _ => must!(text_token!("://")),
         (proto)
     )
 );

make_fn!(domain<StrIter, &str>,
     until!(either!(
         discard!(text_token!("/")),
         discard!(ascii_ws),
         eoi))
 );

make_fn!(path<StrIter, &str>,
      until!(either!(discard!(ascii_ws), eoi))
 );

make_fn!(pub url<StrIter, (Option<&str>, Option<&str>, &str)>,
     do_each!(
         protocol => optional!(proto),
         domain => optional!(domain),
         path => path,
         (protocol, domain, path)
     )
 );

#[test]
fn test_url_parser() {
    let iter = StrIter::new("http://example.com/some/path ");
    let result = url(iter);
    assert!(result.is_complete());
    if let Result::Complete(_, (proto, domain, path)) = result {
        assert!(proto.is_some());
        assert!(domain.is_some());
        assert_eq!(path, "/some/path");
    }
}
