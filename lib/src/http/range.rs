//! Types that map to HTTP Range Requests.

use pear::ParseError;
use smallvec::SmallVec;

use http::{Header, ParseableHeader};
use http::parse::parse_range;
use ext::IntoCollection;

pub enum AcceptRanges {
    Bytes,
    None
}

impl Into<Header<'static>> for AcceptRanges {
    fn into(self) -> Header<'static> {
        const NAME: &str = "Accept-Ranges";

        match self {
            AcceptRanges::Bytes => Header::new(NAME, "bytes"),
            AcceptRanges::None => Header::new(NAME, "none")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RangeSpec {
    From(u64),
    Last(u64),
    Full(u64, u64)
}

impl RangeSpec {
    pub fn within(&self, k: u64) -> bool {
        match *self {
            RangeSpec::From(n) | RangeSpec::Last(n) | RangeSpec::Full(_, n) => n < k,
        }
    }

    pub fn bounds(&self, length: u64) -> (u64, u64) {
        match *self {
            RangeSpec::From(i) => (i, length - i),
            RangeSpec::Last(k) => (length - k, k),
            RangeSpec::Full(i, j) => (i, j - i + 1)
        }
    }
}

// We only support `Bytes` ranges for now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    Bytes(SmallVec<[RangeSpec; 2]>)
}

impl Range {
    pub fn new<R: IntoCollection<RangeSpec>>(ranges: R) -> Range {
        Range::Bytes(ranges.into_collection())
    }
}

use std::ops::Deref;

impl Deref for Range {
    type Target = [RangeSpec];

    fn deref(&self) -> &[RangeSpec] {
        match *self {
            Range::Bytes(ref vec) => vec
        }
    }
}

impl<'h> ParseableHeader<'h> for Range {
    const HEADER_NAME: &'static str = "Range";

    type Error = ParseError<&'h str>;

    fn parse(value: &'h str) -> Result<Self, Self::Error> {
        parse_range(value)
    }
}

pub enum ContentRange {
    // from, to, length
    Bytes(u64, u64, Option<u64>),
    // length
    UnsatisfiableBytes(u64)
}

impl Into<Header<'static>> for ContentRange {
    fn into(self) -> Header<'static> {
        const NAME: &str = "Content-Range";

        match self {
            ContentRange::Bytes(from, to, Some(length)) => {
                Header::new(NAME, format!("bytes {}-{}/{}", from, to, length))
            }
            ContentRange::Bytes(from, to, None) => {
                Header::new(NAME, format!("bytes {}-{}/*", from, to))
            }
            ContentRange::UnsatisfiableBytes(length) => {
                Header::new(NAME, format!("bytes */{}", length))
            }
        }
    }
}

