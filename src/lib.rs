#![deny(missing_docs)]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

//! jed creates Json iterators over instances of io.Read

extern crate rustc_serialize;

use std::io::{ Chars, Read };
use std::iter::Iterator;
use rustc_serialize::json::{ Json, Builder };

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

    fn chomp<R: Read>(mut chars: Chars<R>, buf: &mut String) -> Option<Json> { 
      match chars.next() {
        Some(Ok(c)) => {
          buf.push(c);
          match c {
            '}' | ']' => {
              let cbuf = buf.clone();
              match Builder::new(cbuf.chars()).build() {
                Ok(j) => Some(j),
                _ => chomp(chars, buf)
              }
            }, _ =>
              chomp(chars, buf)
          }
        }, _ => None
      }
    }
    let ref mut inner = self.inner;
    chomp(inner.chars(), &mut String::new())
  }
}

#[cfg(test)]
mod tests {
  use super::Iter;
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

  #[bench]
  fn bench_iter(b: &mut Bencher) {
    b.iter(|| Iter::new(BufReader::new("{'foo':'bar'}{'foo':'baz'}".as_bytes())).count())
  }
}
