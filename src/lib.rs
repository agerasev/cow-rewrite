#![no_std]

extern crate alloc;

use alloc::borrow::{Cow, ToOwned};

mod private {
    pub trait Sealed {}
}
use private::Sealed;

pub trait Rewritable: ToOwned + Sealed {
    type Info: Default;
}

pub struct Rewrite<'a, T: Rewritable + ?Sized> {
    input: Cow<'a, T>,
    output: T::Owned,
    copied: bool,
    index: T::Info,
}

/// Check that bytes `a` contains bytes `b` at offset `i`
fn is_bytes_at(a: &[u8], i: usize, b: &[u8]) -> bool {
    let l = b.len();
    unsafe { a.len() >= i + l && a.get_unchecked(i..(i + l)) == b }
}

/// Check that string slice `s` contains char `c` at byte offset `i`
fn is_char_at(s: &str, i: usize, c: char) -> bool {
    let mut d = [0; size_of::<char>()];
    c.encode_utf8(&mut d);
    let l = c.len_utf8();
    let d = unsafe { &d.get_unchecked(0..l) };
    is_bytes_at(s.as_bytes(), i, d)
}

impl Sealed for str {}
impl Rewritable for str {
    type Info = usize;
}

impl<'a, T: Rewritable<Owned: Default> + ?Sized> Rewrite<'a, T> {
    pub fn new(input: Cow<'a, T>) -> Self {
        Self {
            input,
            output: T::Owned::default(),
            copied: false,
            index: T::Info::default(),
        }
    }
}

impl<'a> Rewrite<'a, str> {
    fn copy(&mut self) {
        if !self.copied {
            self.output.push_str(&self.input[0..self.index]);
            self.copied = true;
        }
    }
    pub fn push(&mut self, c: char) {
        if !self.copied && is_char_at(&self.input, self.index, c) {
            self.index += c.len_utf8();
        } else {
            self.copy();
            self.output.push(c);
        }
    }
    pub fn push_str(&mut self, s: &str) {
        if !self.copied && is_bytes_at(self.input.as_bytes(), self.index, s.as_bytes()) {
            self.index += s.len();
        } else {
            self.copy();
            self.output.push_str(s);
        }
    }
}

impl<'a> From<Rewrite<'a, str>> for Cow<'a, str> {
    fn from(this: Rewrite<'a, str>) -> Self {
        if this.copied {
            Cow::Owned(this.output)
        } else {
            match this.input {
                Cow::Borrowed(s) => Cow::Borrowed(&s[0..this.index]),
                Cow::Owned(mut s) => {
                    s.truncate(this.index);
                    Cow::Owned(s)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_write_same() {
        let s = "abc";
        let mut r = Rewrite::new(Cow::Borrowed(s));
        r.push('a');
        r.push_str("bc");
        let d: Cow<str> = r.into();
        assert_eq!(d, Cow::Borrowed(s))
    }

    #[test]
    fn str_write_less() {
        let s = "abc";
        let mut r = Rewrite::new(Cow::Borrowed(s));
        r.push('a');
        r.push_str("b");
        let d: Cow<str> = r.into();
        assert_eq!(d, Cow::Borrowed(&s[..2]))
    }

    #[test]
    fn str_write_more() {
        let s = "abc";
        let mut r = Rewrite::new(Cow::Borrowed(s));
        r.push('a');
        r.push_str("bcd");
        let d: Cow<str> = r.into();
        assert_eq!(d, Cow::<str>::Owned("abcd".to_owned()))
    }

    #[test]
    fn str_write_different() {
        let s = "abc";
        let mut r = Rewrite::new(Cow::Borrowed(s));
        r.push('a');
        r.push_str("bd");
        let d: Cow<str> = r.into();
        assert_eq!(d, Cow::<str>::Owned("abd".to_owned()))
    }
}
