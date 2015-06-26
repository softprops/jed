//#![deny(missing_docs)]
//#![cfg_attr(all(test, feature = "nightly"), feature(test))]
//#![cfg_attr(all(feature = "nightly"), feature(io))]

//! jed creates Json iterators over instances of io.Read

extern crate rustc_serialize;

use std::io::{ /*Chars,*/ Read };
use std::iter::Iterator;
use rustc_serialize::json::{ Json, Builder };

// workaround imports for std::io::Read::chars()
use std::error::Error;
use std::{ fmt, io, result, str };

/// An iterator over the Json elements of an io::Read stream
pub struct Iter<R> {
  inner: R
}

impl<R: Read> Iter<R> {
  /// Create a new Iter instance
  pub fn new(inner: R) -> Iter<R> {
    Iter { inner: inner }
  }
}

impl<R: Read> Iterator for Iter<R> {
  type Item = Json;

  fn next(&mut self) -> Option<Json> {
    let ref mut inner = self.inner;
    let mut chars = Chars { inner: inner };
    let mut buf = String::new();
    while let Some(Ok(c)) = chars.next() {
      buf.push(c);
      match c {
        '}' | ']' =>
          match Builder::new(buf.chars()).build() {
            Ok(j) => return Some(j),
            _ => ()
          },
        _ => ()
      }
    }
    None
  }
}

/// work arounds until read::chars() stablizes

static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

#[inline]
fn utf8_char_width(b: u8) -> usize {
    return UTF8_CHAR_WIDTH[b as usize] as usize;
}

struct Chars<R> {
  inner: R,
}

#[derive(Debug)]
enum CharsError {
  NotUtf8,
  Other(io::Error),
}

impl<R: Read> Iterator for Chars<R> {
  type Item = result::Result<char, CharsError>;

  fn next(&mut self) -> Option<result::Result<char, CharsError>> {
    let mut buf = [0];
    let first_byte = match self.inner.read(&mut buf) {
      Ok(0) => return None,
      Ok(..) => buf[0],
      Err(e) => return Some(Err(CharsError::Other(e))),
    };
    let width = utf8_char_width(first_byte);
    if width == 1 { return Some(Ok(first_byte as char)) }
    if width == 0 { return Some(Err(CharsError::NotUtf8)) }
    let mut buf = [first_byte, 0, 0, 0];
    {
      let mut start = 1;
      while start < width {
        match self.inner.read(&mut buf[start..width]) {
          Ok(0) => return Some(Err(CharsError::NotUtf8)),
          Ok(n) => start += n,
          Err(e) => return Some(Err(CharsError::Other(e))),
        }
      }
    }
    Some(match str::from_utf8(&buf[..width]).ok() {
      Some(s) => {
        let v: Vec<char> = s.chars().collect();
        Ok(v[0])
      },
      None => Err(CharsError::NotUtf8),
    })
  }
}

impl Error for CharsError {
  fn description(&self) -> &str {
    match *self {
      CharsError::NotUtf8 => "invalid utf8 encoding",
      CharsError::Other(ref e) => Error::description(e),
    }
  }
  fn cause(&self) -> Option<&Error> {
    match *self {
      CharsError::NotUtf8 => None,
      CharsError::Other(ref e) => e.cause(),
    }
  }
}

impl fmt::Display for CharsError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      CharsError::NotUtf8 => {
        "byte stream did not contain valid utf8".fmt(f)
      }
      CharsError::Other(ref e) => e.fmt(f),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Iter;
  #[cfg(feature = "nightly")]
  use test::Bencher;
  use std::io::{ empty, BufReader };

  #[test]
  fn test_not_json_iter() {
    let reader = BufReader::new("bogus".as_bytes());
    assert_eq!(Iter::new(reader).count(), 0);
  }

  #[test]
  fn test_empty_iter() {
    assert_eq!(Iter::new(empty()).count(), 0);
  }

  #[test]
  fn test_ary_iter() {
    let reader = BufReader::new("[][]".as_bytes());
    assert_eq!(Iter::new(reader).count(), 2)
  }

  #[test]
  fn test_obj_iter() {
    let reader = BufReader::new("{}{}".as_bytes());
    assert_eq!(Iter::new(reader).count(), 2)
  }

  #[cfg(feature = "nightly")]
  #[bench]
  fn bench_iter(b: &mut Bencher) {
    b.iter(|| Iter::new(BufReader::new("{'foo':'bar'}{'foo':'baz'}".as_bytes())).count())
  }
}
