use pear::{ParseError, ParseResult};
use pear::parsers::*;
use smallvec::SmallVec;

use http::range::{Range, RangeSpec};
use http::parse::checkers::{is_digit, is_whitespace as is_ws};

#[parser]
pub fn range<'a>(input: &mut &'a str) -> ParseResult<&'a str, Range> {
    let mut ranges = SmallVec::new();

    (eat_slice("bytes="), ranges.push(range_spec()), skip_while(is_ws));
    switch_repeat! {
        eat(',') => {
            (skip_while(is_ws), ranges.push(range_spec()), skip_while(is_ws))
        }
    }

    Range::Bytes(ranges)
}

#[parser]
pub fn index<'a>(input: &mut &'a str) -> ParseResult<&'a str, u64> {
    from!(take_some_while(is_digit).parse())
}

#[parser]
pub fn range_spec<'a>(input: &mut &'a str) -> ParseResult<&'a str, RangeSpec> {
    let (from, _, to) = (maybe!(index()), eat('-'), maybe!(index()));
    let spec = match (from, to) {
        (Some(from), Some(to)) if from <= to => RangeSpec::Full(from, to),
        (None, Some(last)) => RangeSpec::Last(last),
        (Some(from), None) => RangeSpec::From(from),
        (Some(_), Some(_)) => parse_error!("range_spec", "from >= to"),
        (None, None) => parse_error!("range_spec", "expected from or to in range")
    };

    spec
}

pub fn parse_range(mut input: &str) -> Result<Range, ParseError<&str>> {
    parse!(&mut input, (range(), eof()).0).into()
}

#[cfg(test)]
mod parse_range_tests {
    use http::range::{Range, RangeSpec};
    use super::parse_range;

    macro_rules! assert_no_parse {
        ($string:expr) => ({
            let result: Result<_, _> = parse_range($string).into();
            if result.is_ok() {
                panic!("{:?} parsed unexpectedly.", $string)
            }
        });
    }

    macro_rules! assert_parse {
        ($string:expr) => ({
            match parse_range($string) {
                Ok(range) => range,
                Err(e) => panic!("{:?} failed to parse: {}", $string, e)
            }
        });
    }

    macro_rules! assert_parse_eq {
        ($string:expr, $result:expr) => (assert_eq!($result, assert_parse!($string)));
    }

    #[test]
    fn check_parse_eq() {
        assert_parse_eq!("bytes=100-200", Range::new(RangeSpec::Full(100, 200)));
        assert_parse_eq!("bytes=100-", Range::new(RangeSpec::From(100)));
        assert_parse_eq!("bytes=-239", Range::new(RangeSpec::Last(239)));
        assert_parse_eq!("bytes=1-3", Range::new(RangeSpec::Full(1, 3)));
        assert_parse_eq!("bytes=1-1", Range::new(RangeSpec::Full(1, 1)));
        assert_parse_eq!("bytes=0-13", Range::new(RangeSpec::Full(0, 13)));

        assert_parse_eq!("bytes=1-3 ", Range::new(RangeSpec::Full(1, 3)));
        assert_parse_eq!("bytes=1-3   ", Range::new(RangeSpec::Full(1, 3)));

        assert_parse_eq!("bytes=1-2,3-3", Range::new(&[
            RangeSpec::Full(1, 2), RangeSpec::Full(3, 3)
        ]));
        assert_parse_eq!("bytes=1-2,3-", Range::new(&[
            RangeSpec::Full(1, 2), RangeSpec::From(3)
        ]));
        assert_parse_eq!("bytes=-2,3-5", Range::new(&[
            RangeSpec::Last(2), RangeSpec::Full(3, 5)
        ]));
        assert_parse_eq!("bytes=-2, 3-5", Range::new(&[
            RangeSpec::Last(2), RangeSpec::Full(3, 5)
        ]));
        assert_parse_eq!("bytes=-2 , 3-5", Range::new(&[
            RangeSpec::Last(2), RangeSpec::Full(3, 5)
        ]));
        assert_parse_eq!("bytes=3-,1-2", Range::new(&[
            RangeSpec::From(3), RangeSpec::Full(1, 2)
        ]));
        assert_parse_eq!("bytes=1-2,2-3,3-4,6-,-10", Range::new(&[
            RangeSpec::Full(1, 2), RangeSpec::Full(2, 3), RangeSpec::Full(3, 4),
            RangeSpec::From(6), RangeSpec::Last(10)
        ]));
        assert_parse_eq!("bytes=1-2  ,2-3 , 3-4,  6-  ,-10", Range::new(&[
            RangeSpec::Full(1, 2), RangeSpec::Full(2, 3), RangeSpec::Full(3, 4),
            RangeSpec::From(6), RangeSpec::Last(10)
        ]));
    }

    #[test]
    fn test_bad_parses() {
        assert_no_parse!("bytes=");
        assert_no_parse!("range=100-200");
        assert_no_parse!("bytes 100-200");
        assert_no_parse!("bytes-100-200");
        assert_no_parse!("100-200");
        assert_no_parse!("bytes=100");
        assert_no_parse!("bytes=-");
        assert_no_parse!("byte=100-200");
        assert_no_parse!("bytes=100-200-");
        assert_no_parse!("bytes=100-200-300");
        assert_no_parse!("bytes=a-b");
        assert_no_parse!("bytes=100a-b");
        assert_no_parse!("bytes=a100-b");
        assert_no_parse!("bytes=a-100");
        assert_no_parse!("bytes=100-a");
        assert_no_parse!("bytes=a-");
        assert_no_parse!("bytes=-a");
        assert_no_parse!("bytes= 100-200");

        assert_no_parse!("bytes=3-1");
        assert_no_parse!("");
    }
}
