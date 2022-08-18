#![cfg_attr(rustfmt, rustfmt_skip)]

use core::fmt;
use core::ops::{Index, IndexMut};
use core::ops::{Range, RangeTo, RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};
use core::slice::{self, Iter, IterMut, SliceIndex};
#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::ffi::CStr;

use ascii_char::AsciiChar;
#[cfg(feature = "std")]
use ascii_string::AsciiString;

/// AsciiStr represents a byte or string slice that only contains ASCII characters.
///
/// It wraps an `[AsciiChar]` and implements many of `str`s methods and traits.
///
/// It can be created by a checked conversion from a `str` or `[u8]`, or borrowed from an
/// `AsciiString`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AsciiStr {
    slice: [AsciiChar],
}

impl AsciiStr {
    /// Converts `&self` to a `&str` slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { &*(self as *const AsciiStr as *const str) }
    }

    /// Converts `&self` into a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { &*(self as *const AsciiStr as *const[u8]) }
    }

    /// Returns the entire string as slice of `AsciiChar`s.
    #[inline]
    pub const fn as_slice(&self) -> &[AsciiChar] {
        &self.slice
    }

    /// Returns the entire string as mutable slice of `AsciiChar`s.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [AsciiChar] {
        &mut self.slice
    }

    /// Returns a raw pointer to the `AsciiStr`'s buffer.
    ///
    /// The caller must ensure that the slice outlives the pointer this function returns, or else it
    /// will end up pointing to garbage. Modifying the `AsciiStr` may cause it's buffer to be
    /// reallocated, which would also make any pointers to it invalid.
    #[inline]
    pub const fn as_ptr(&self) -> *const AsciiChar {
        self.as_slice().as_ptr()
    }

    /// Returns an unsafe mutable pointer to the `AsciiStr`'s buffer.
    ///
    /// The caller must ensure that the slice outlives the pointer this function returns, or else it
    /// will end up pointing to garbage. Modifying the `AsciiStr` may cause it's buffer to be
    /// reallocated, which would also make any pointers to it invalid.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut AsciiChar {
        self.as_mut_slice().as_mut_ptr()
    }

    /// Copies the content of this `AsciiStr` into an owned `AsciiString`.
    #[cfg(feature = "std")]
    pub fn to_ascii_string(&self) -> AsciiString {
        AsciiString::from(self.slice.to_vec())
    }

    /// Converts anything that can represent a byte slice into an `AsciiStr`.
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let foo = AsciiStr::from_ascii(b"foo");
    /// let err = AsciiStr::from_ascii("Ŋ");
    /// assert_eq!(foo.unwrap().as_str(), "foo");
    /// assert_eq!(err.unwrap_err().valid_up_to(), 0);
    /// ```
    #[inline]
    pub fn from_ascii<B: ?Sized>(bytes: &B) -> Result<&AsciiStr, AsAsciiStrError>
    where
        B: AsRef<[u8]>,
    {
        bytes.as_ref().as_ascii_str()
    }

    /// Converts anything that can be represented as a byte slice to an `AsciiStr` without checking
    /// for non-ASCII characters..
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let foo = unsafe { AsciiStr::from_ascii_unchecked(&b"foo"[..]) };
    /// assert_eq!(foo.as_str(), "foo");
    /// ```
    #[inline]
    pub unsafe fn from_ascii_unchecked(bytes: &[u8]) -> &AsciiStr {
        bytes.as_ascii_str_unchecked()
    }

    /// Returns the number of characters / bytes in this ASCII sequence.
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let s = AsciiStr::from_ascii("foo").unwrap();
    /// assert_eq!(s.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns true if the ASCII slice contains zero bytes.
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let mut empty = AsciiStr::from_ascii("").unwrap();
    /// let mut full = AsciiStr::from_ascii("foo").unwrap();
    /// assert!(empty.is_empty());
    /// assert!(!full.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the characters of the `AsciiStr`.
    #[inline]
    pub fn chars(&self) -> Chars {
        Chars(self.slice.iter())
    }

    /// Returns an iterator over the characters of the `AsciiStr` which allows you to modify the
    /// value of each `AsciiChar`.
    #[inline]
    pub fn chars_mut(&mut self) -> CharsMut {
        CharsMut(self.slice.iter_mut())
    }

    /// Returns an iterator over parts of the `AsciiStr` separated by a character.
    ///
    /// # Examples
    /// ```
    /// # use ascii::{AsciiStr, AsciiChar};
    /// let words = AsciiStr::from_ascii("apple banana lemon").unwrap()
    ///     .split(AsciiChar::Space)
    ///     .map(|a| a.as_str())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(words, ["apple", "banana", "lemon"]);
    /// ```
    pub fn split(&self, on: AsciiChar) -> impl DoubleEndedIterator<Item=&AsciiStr> {
        Split {
            on,
            ended: false,
            chars: self.chars(),
        }
    }

    /// Returns an iterator over the lines of the `AsciiStr`, which are themselves `AsciiStr`s.
    ///
    /// Lines are ended with either `LineFeed` (`\n`), or `CarriageReturn` then `LineFeed` (`\r\n`).
    ///
    /// The final line ending is optional.
    #[inline]
    pub fn lines(&self) -> impl DoubleEndedIterator<Item=&AsciiStr> {
        Lines {
            string: self,
        }
    }

    /// Returns an ASCII string slice with leading and trailing whitespace removed.
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let example = AsciiStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("white \tspace", example.trim());
    /// ```
    pub fn trim(&self) -> &Self {
        self.trim_start().trim_end()
    }

    /// Returns an ASCII string slice with leading whitespace removed.
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let example = AsciiStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("white \tspace  \t", example.trim_start());
    /// ```
    pub fn trim_start(&self) -> &Self {
        &self[self.chars().take_while(|a| a.is_whitespace()).count()..]
    }

    /// Returns an ASCII string slice with trailing whitespace removed.
    ///
    /// # Examples
    /// ```
    /// # use ascii::AsciiStr;
    /// let example = AsciiStr::from_ascii("  \twhite \tspace  \t").unwrap();
    /// assert_eq!("  \twhite \tspace", example.trim_end());
    /// ```
    pub fn trim_end(&self) -> &Self {
        let trimmed = self.chars()
            .rev()
            .take_while(|a| a.is_whitespace())
            .count();
        &self[..self.len() - trimmed]
    }

    /// Compares two strings case-insensitively.
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.len() == other.len() &&
            self.chars().zip(other.chars()).all(|(a, b)| {
                a.eq_ignore_ascii_case(&b)
            })
    }

    /// Replaces lowercase letters with their uppercase equivalent.
    pub fn make_ascii_uppercase(&mut self) {
        for a in self.chars_mut() {
            *a = a.to_ascii_uppercase();
        }
    }

    /// Replaces uppercase letters with their lowercase equivalent.
    pub fn make_ascii_lowercase(&mut self) {
        for a in self.chars_mut() {
            *a = a.to_ascii_lowercase();
        }
    }

    /// Returns a copy of this string where letters 'a' to 'z' are mapped to 'A' to 'Z'.
    #[cfg(feature="std")]
    pub fn to_ascii_uppercase(&self) -> AsciiString {
        let mut ascii_string = self.to_ascii_string();
        ascii_string.make_ascii_uppercase();
        ascii_string
    }

    /// Returns a copy of this string where letters 'A' to 'Z' are mapped to 'a' to 'z'.
    #[cfg(feature="std")]
    pub fn to_ascii_lowercase(&self) -> AsciiString {
        let mut ascii_string = self.to_ascii_string();
        ascii_string.make_ascii_lowercase();
        ascii_string
    }

    /// Returns the first character if the string is not empty.
    #[inline]
    pub fn first(&self) -> Option<AsciiChar> {
        self.slice.first().cloned()
    }

    /// Returns the last character if the string is not empty.
    #[inline]
    pub fn last(&self) -> Option<AsciiChar> {
        self.slice.last().cloned()
    }
}

macro_rules! impl_partial_eq {
    ($wider: ty) => {
        impl PartialEq<$wider> for AsciiStr {
            #[inline]
            fn eq(&self, other: &$wider) -> bool {
                <AsciiStr as AsRef<$wider>>::as_ref(self) == other
            }
        }
        impl PartialEq<AsciiStr> for $wider {
            #[inline]
            fn eq(&self, other: &AsciiStr) -> bool {
                self == <AsciiStr as AsRef<$wider>>::as_ref(other)
            }
        }
    };
}

impl_partial_eq!{str}
impl_partial_eq!{[u8]}
impl_partial_eq!{[AsciiChar]}

#[cfg(feature = "std")]
impl ToOwned for AsciiStr {
    type Owned = AsciiString;

    #[inline]
    fn to_owned(&self) -> AsciiString {
        self.to_ascii_string()
    }
}

impl AsRef<[u8]> for AsciiStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl AsRef<str> for AsciiStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<[AsciiChar]> for AsciiStr {
    #[inline]
    fn as_ref(&self) -> &[AsciiChar] {
        &self.slice
    }
}
impl AsMut<[AsciiChar]> for AsciiStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [AsciiChar] {
        &mut self.slice
    }
}

impl Default for &'static AsciiStr {
    #[inline]
    fn default() -> &'static AsciiStr {
        From::from(&[] as &[AsciiChar])
    }
}
impl<'a> From<&'a [AsciiChar]> for &'a AsciiStr {
    #[inline]
    fn from(slice: &[AsciiChar]) -> &AsciiStr {
        let ptr = slice as *const [AsciiChar] as *const AsciiStr;
        unsafe { &*ptr }
    }
}
impl<'a> From<&'a mut [AsciiChar]> for &'a mut AsciiStr {
    #[inline]
    fn from(slice: &mut [AsciiChar]) -> &mut AsciiStr {
        let ptr = slice as *mut [AsciiChar] as *mut AsciiStr;
        unsafe { &mut *ptr }
    }
}
#[cfg(feature = "std")]
impl From<Box<[AsciiChar]>> for Box<AsciiStr> {
    #[inline]
    fn from(owned: Box<[AsciiChar]>) -> Box<AsciiStr> {
        let ptr = Box::into_raw(owned) as *mut AsciiStr;
        unsafe { Box::from_raw(ptr) }
    }
}

impl AsRef<AsciiStr> for AsciiStr {
    #[inline]
    fn as_ref(&self) -> &AsciiStr {
        self
    }
}
impl AsMut<AsciiStr> for AsciiStr {
    #[inline]
    fn as_mut(&mut self) -> &mut AsciiStr {
        self
    }
}
impl AsRef<AsciiStr> for [AsciiChar] {
    #[inline]
    fn as_ref(&self) -> &AsciiStr {
        self.into()
    }
}
impl AsMut<AsciiStr> for [AsciiChar] {
    #[inline]
    fn as_mut(&mut self) -> &mut AsciiStr {
        self.into()
    }
}

impl<'a> From<&'a AsciiStr> for &'a [AsciiChar] {
    #[inline]
    fn from(astr: &AsciiStr) -> &[AsciiChar] {
        &astr.slice
    }
}
impl<'a> From<&'a mut AsciiStr> for &'a mut [AsciiChar] {
    #[inline]
    fn from(astr: &mut AsciiStr) -> &mut [AsciiChar] {
        &mut astr.slice
    }
}
impl<'a> From<&'a AsciiStr> for &'a [u8] {
    #[inline]
    fn from(astr: &AsciiStr) -> &[u8] {
        astr.as_bytes()
    }
}
impl<'a> From<&'a AsciiStr> for &'a str {
    #[inline]
    fn from(astr: &AsciiStr) -> &str {
        astr.as_str()
    }
}
macro_rules! widen_box {
    ($wider: ty) => {
        #[cfg(feature = "std")]
        impl From<Box<AsciiStr>> for Box<$wider> {
            #[inline]
            fn from(owned: Box<AsciiStr>) -> Box<$wider> {
                let ptr = Box::into_raw(owned) as *mut $wider;
                unsafe { Box::from_raw(ptr) }
            }
        }
    }
}
widen_box! {[AsciiChar]}
widen_box! {[u8]}
widen_box! {str}

// allows &AsciiChar to be used by generic AsciiString Extend and FromIterator
impl AsRef<AsciiStr> for AsciiChar {
    fn as_ref(&self) -> &AsciiStr {
        slice::from_ref(self).into()
    }
}

impl fmt::Display for AsciiStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Debug for AsciiStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

macro_rules! impl_index {
    ($idx:ty) => {
        impl Index<$idx> for AsciiStr {
            type Output = AsciiStr;

            #[inline]
            fn index(&self, index: $idx) -> &AsciiStr {
                self.slice[index].as_ref()
            }
        }

        impl IndexMut<$idx> for AsciiStr {
            #[inline]
            fn index_mut(&mut self, index: $idx) -> &mut AsciiStr {
                self.slice[index].as_mut()
            }
        }
    }
}

impl_index! { Range<usize> }
impl_index! { RangeTo<usize> }
impl_index! { RangeFrom<usize> }
impl_index! { RangeFull }
impl_index! { RangeInclusive<usize> }
impl_index! { RangeToInclusive<usize> }

impl Index<usize> for AsciiStr {
    type Output = AsciiChar;

    #[inline]
    fn index(&self, index: usize) -> &AsciiChar {
        &self.slice[index]
    }
}

impl IndexMut<usize> for AsciiStr {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut AsciiChar {
        &mut self.slice[index]
    }
}

/// Produces references for compatibility with `[u8]`.
///
/// (`str` doesn't implement `IntoIterator` for its references,
///  so there is no compatibility to lose.)
impl<'a> IntoIterator for &'a AsciiStr {
    type Item = &'a AsciiChar;
    type IntoIter = CharsRef<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CharsRef(self.as_slice().iter())
    }
}

impl<'a> IntoIterator for &'a mut AsciiStr {
    type Item = &'a mut AsciiChar;
    type IntoIter = CharsMut<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.chars_mut()
    }
}

/// A copying iterator over the characters of an `AsciiStr`.
#[derive(Clone, Debug)]
pub struct Chars<'a>(Iter<'a, AsciiChar>);
impl<'a> Chars<'a> {
    /// Returns the ascii string slice with the remaining characters.
    pub fn as_str(&self) -> &'a AsciiStr {
        self.0.as_slice().into()
    }
}
impl<'a> Iterator for Chars<'a> {
    type Item = AsciiChar;
    #[inline]
    fn next(&mut self) -> Option<AsciiChar> {
        self.0.next().cloned()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<AsciiChar> {
        self.0.next_back().cloned()
    }
}
impl<'a> ExactSizeIterator for Chars<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A mutable iterator over the characters of an `AsciiStr`.
#[derive(Debug)]
pub struct CharsMut<'a>(IterMut<'a, AsciiChar>);
impl<'a> CharsMut<'a> {
    /// Returns the ascii string slice with the remaining characters.
    pub fn into_str(self) -> &'a mut AsciiStr {
        self.0.into_slice().into()
    }
}
impl<'a> Iterator for CharsMut<'a> {
    type Item = &'a mut AsciiChar;
    #[inline]
    fn next(&mut self) -> Option<&'a mut AsciiChar> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> DoubleEndedIterator for CharsMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut AsciiChar> {
        self.0.next_back()
    }
}
impl<'a> ExactSizeIterator for CharsMut<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// An immutable iterator over the characters of an `AsciiStr`.
#[derive(Clone, Debug)]
pub struct CharsRef<'a>(Iter<'a, AsciiChar>);
impl<'a> CharsRef<'a> {
    /// Returns the ascii string slice with the remaining characters.
    pub fn as_str(&self) -> &'a AsciiStr {
        self.0.as_slice().into()
    }
}
impl<'a> Iterator for CharsRef<'a> {
    type Item = &'a AsciiChar;
    #[inline]
    fn next(&mut self) -> Option<&'a AsciiChar> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> DoubleEndedIterator for CharsRef<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a AsciiChar> {
        self.0.next_back()
    }
}

/// An iterator over parts of an `AsciiStr` separated by an `AsciiChar`.
///
/// This type is created by [`AsciiChar::split()`](struct.AsciiChar.html#method.split).
#[derive(Clone, Debug)]
struct Split<'a> {
    on: AsciiChar,
    ended: bool,
    chars: Chars<'a>
}
impl<'a> Iterator for Split<'a> {
    type Item = &'a AsciiStr;

    fn next(&mut self) -> Option<&'a AsciiStr> {
        if !self.ended {
            let start: &AsciiStr = self.chars.as_str();
            let split_on = self.on;
            if let Some(at) = self.chars.position(|c| c == split_on) {
                Some(&start[..at])
            } else {
                self.ended = true;
                Some(start)
            }
        } else {
            None
        }
    }
}
impl<'a> DoubleEndedIterator for Split<'a> {
    fn next_back(&mut self) -> Option<&'a AsciiStr> {
        if !self.ended {
            let start: &AsciiStr = self.chars.as_str();
            let split_on = self.on;
            if let Some(at) = self.chars.rposition(|c| c == split_on) {
                Some(&start[at+1..])
            } else {
                self.ended = true;
                Some(start)
            }
        } else {
            None
        }
    }
}

/// An iterator over the lines of the internal character array.
#[derive(Clone, Debug)]
struct Lines<'a> {
    string: &'a AsciiStr,
}
impl<'a> Iterator for Lines<'a> {
    type Item = &'a AsciiStr;

    fn next(&mut self) -> Option<&'a AsciiStr> {
        if let Some(idx) = self.string
            .chars()
            .position(|chr| chr == AsciiChar::LineFeed)
        {
            let line = if idx > 0 && self.string[idx - 1] == AsciiChar::CarriageReturn {
                &self.string[..idx - 1]
            } else {
                &self.string[..idx]
            };
            self.string = &self.string[idx + 1..];
            Some(line)
        } else if self.string.is_empty() {
            None
        } else {
            let line = self.string;
            self.string = &self.string[..0];
            Some(line)
        }
    }
}
impl<'a> DoubleEndedIterator for Lines<'a> {
    fn next_back(&mut self) -> Option<&'a AsciiStr> {
        if self.string.is_empty() {
            return None;
        }
        let mut i = self.string.len();
        if self.string[i-1] == AsciiChar::LineFeed {
            i -= 1;
            if i > 0 && self.string[i-1] == AsciiChar::CarriageReturn {
                i -= 1;
            }
        }
        self.string = &self.string[..i];
        while i > 0 && self.string[i-1] != AsciiChar::LineFeed {
            i -= 1;
        }
        let line = &self.string[i..];
        self.string = &self.string[..i];
        Some(line)
    }
}

/// Error that is returned when a sequence of `u8` are not all ASCII.
///
/// Is used by `As[Mut]AsciiStr` and the `from_ascii` method on `AsciiStr` and `AsciiString`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AsAsciiStrError(usize);

const ERRORMSG_STR: &str = "one or more bytes are not ASCII";

impl AsAsciiStrError {
    /// Returns the index of the first non-ASCII byte.
    ///
    /// It is the maximum index such that `from_ascii(input[..index])` would return `Ok(_)`.
    #[inline]
    pub const fn valid_up_to(self) -> usize {
        self.0
    }
    #[cfg(not(feature = "std"))]
    /// Returns a description for this error, like `std::error::Error::description`.
    #[inline]
    pub const fn description(&self) -> &'static str {
        ERRORMSG_STR
    }
}
impl fmt::Display for AsAsciiStrError {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "the byte at index {} is not ASCII", self.0)
    }
}
#[cfg(feature = "std")]
impl Error for AsAsciiStrError {
    #[inline]
    fn description(&self) -> &'static str {
        ERRORMSG_STR
    }
}

/// Convert slices of bytes or AsciiChar to `AsciiStr`.
// Could nearly replace this trait with SliceIndex, but its methods isn't even
// on a path for stabilization.
pub trait AsAsciiStr {
    /// Used to constrain `SliceIndex`
    #[doc(hidden)]
    type Inner;
    /// Convert a subslice to an ASCII slice.
    ///
    /// Returns `Err` if the range is out of bounds or if not all bytes in the
    /// slice are ASCII. The value in the error will be the index of the first
    /// non-ASCII byte or the end of the slice.
    ///
    /// # Examples
    /// ```
    /// use ascii::AsAsciiStr;
    /// assert!("'zoä'".slice_ascii(..3).is_ok());
    /// assert!("'zoä'".slice_ascii(0..4).is_err());
    /// assert!("'zoä'".slice_ascii(5..=5).is_ok());
    /// assert!("'zoä'".slice_ascii(4..).is_err());
    /// assert!(b"\r\n".slice_ascii(..).is_ok());
    /// ```
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[Self::Inner], Output=[Self::Inner]>;
    /// Convert to an ASCII slice.
    ///
    /// # Example
    /// ```
    /// use ascii::{AsAsciiStr, AsciiChar};
    /// assert!("ASCII".as_ascii_str().is_ok());
    /// assert!(b"\r\n".as_ascii_str().is_ok());
    /// assert!("'zoä'".as_ascii_str().is_err());
    /// assert!(b"\xff".as_ascii_str().is_err());
    /// assert!([AsciiChar::C][..].as_ascii_str().is_ok()); // infallible
    /// ```
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        self.slice_ascii(..)
    }
    /// Get a single ASCII character from the slice.
    ///
    /// Returns `None` if the index is out of bounds or the byte is not ASCII.
    ///
    /// # Examples
    /// ```
    /// use ascii::{AsAsciiStr, AsciiChar};
    /// assert_eq!("'zoä'".get_ascii(4), None);
    /// assert_eq!("'zoä'".get_ascii(5), Some(AsciiChar::Apostrophe));
    /// assert_eq!("'zoä'".get_ascii(6), None);
    /// ```
    fn get_ascii(&self, index: usize) -> Option<AsciiChar> {
        self.slice_ascii(index..=index).ok().and_then(|str| str.first())
    }
    /// Convert to an ASCII slice without checking for non-ASCII characters.
    ///
    /// # Examples
    /// 
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr;
}

/// Convert mutable slices of bytes or AsciiChar to `AsciiStr`.
pub trait AsMutAsciiStr: AsAsciiStr {
    /// Convert a subslice to an ASCII slice.
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[Self::Inner], Output=[Self::Inner]>;
    /// Convert to a mutable ASCII slice.
    fn as_mut_ascii_str(&mut self) -> Result<&mut AsciiStr, AsAsciiStrError> {
        self.slice_ascii_mut(..)
    }
    /// Convert to a mutable ASCII slice without checking for non-ASCII characters.
    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr;
}

// These generic implementations mirror the generic implementations for AsRef<T> in core.
impl<'a, T: ?Sized> AsAsciiStr for &'a T where T: AsAsciiStr {
    type Inner = <T as AsAsciiStr>::Inner;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[Self::Inner], Output=[Self::Inner]>
    {
        <T as AsAsciiStr>::slice_ascii(*self, range)
    }
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        <T as AsAsciiStr>::as_ascii_str_unchecked(*self)
    }
}

impl<'a, T: ?Sized> AsAsciiStr for &'a mut T where T: AsAsciiStr {
    type Inner = <T as AsAsciiStr>::Inner;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[Self::Inner], Output=[Self::Inner]>
    {
        <T as AsAsciiStr>::slice_ascii(*self, range)
    }

    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        <T as AsAsciiStr>::as_ascii_str_unchecked(*self)
    }
}

impl<'a, T: ?Sized> AsMutAsciiStr for &'a mut T where T: AsMutAsciiStr {
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[Self::Inner], Output=[Self::Inner]>
    {
        <T as AsMutAsciiStr>::slice_ascii_mut(*self, range)
    }

    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr {
        <T as AsMutAsciiStr>::as_mut_ascii_str_unchecked(*self)
    }
}

impl AsAsciiStr for AsciiStr {
    type Inner = AsciiChar;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[AsciiChar], Output=[AsciiChar]>
    {
        self.slice.slice_ascii(range)
    }
    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        Ok(self)
    }
    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        self
    }
    #[inline]
    fn get_ascii(&self, index: usize) -> Option<AsciiChar> {
        self.slice.get_ascii(index)
    }
}
impl AsMutAsciiStr for AsciiStr {
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[AsciiChar], Output=[AsciiChar]>
    {
        self.slice.slice_ascii_mut(range)
    }
    #[inline]
    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr {
        self
    }
}

impl AsAsciiStr for [AsciiChar] {
    type Inner = AsciiChar;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[AsciiChar], Output=[AsciiChar]>
    {
        match self.get(range) {
            Some(slice) => Ok(slice.into()),
            None => Err(AsAsciiStrError(self.len())),
        }
    }
    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        Ok(self.into())
    }
    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        self.into()
    }
    #[inline]
    fn get_ascii(&self, index: usize) -> Option<AsciiChar> {
        self.get(index).cloned()
    }
}
impl AsMutAsciiStr for [AsciiChar] {
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[AsciiChar], Output=[AsciiChar]>
    {
        let len = self.len();
        match self.get_mut(range) {
            Some(slice) => Ok(slice.into()),
            None => Err(AsAsciiStrError(len)),
        }
    }
    #[inline]
    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr {
        self.into()
    }
}

impl AsAsciiStr for [u8] {
    type Inner = u8;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[u8], Output=[u8]>
    {
        if let Some(slice) = self.get(range) {
            slice.as_ascii_str().map_err(|AsAsciiStrError(not_ascii)| {
                let offset = slice.as_ptr() as usize - self.as_ptr() as usize;
                AsAsciiStrError(offset + not_ascii)
            })
        } else {
            Err(AsAsciiStrError(self.len()))
        }
    }
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        if self.is_ascii() {// is_ascii is likely optimized
            unsafe { Ok(self.as_ascii_str_unchecked()) }
        } else {
            Err(AsAsciiStrError(self.iter().take_while(|&b| b.is_ascii()).count()))
        }
    }
    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        let ptr = self as *const [u8] as *const AsciiStr;
        &*ptr
    }
}
impl AsMutAsciiStr for [u8] {
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[u8], Output=[u8]>
    {
        let (ptr, len) = (self.as_ptr(), self.len());
        if let Some(slice) = self.get_mut(range) {
            let slice_ptr = slice.as_ptr();
            slice.as_mut_ascii_str().map_err(|AsAsciiStrError(not_ascii)| {
                let offset = slice_ptr as usize - ptr as usize;
                AsAsciiStrError(offset + not_ascii)
            })
        } else {
            Err(AsAsciiStrError(len))
        }
    }
    fn as_mut_ascii_str(&mut self) -> Result<&mut AsciiStr, AsAsciiStrError> {
        if self.is_ascii() {// is_ascii() is likely optimized
            unsafe { Ok(self.as_mut_ascii_str_unchecked()) }
        } else {
            Err(AsAsciiStrError(self.iter().take_while(|&b| b.is_ascii()).count()))
        }
    }
    #[inline]
    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr {
        let ptr = self as *mut [u8] as *mut AsciiStr;
        &mut *ptr
    }
}

impl AsAsciiStr for str {
    type Inner = u8;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[u8], Output=[u8]>
    {
        self.as_bytes().slice_ascii(range)
    }
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        self.as_bytes().as_ascii_str()
    }
    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        self.as_bytes().as_ascii_str_unchecked()
    }
}
impl AsMutAsciiStr for str {
    fn slice_ascii_mut<R>(&mut self, range: R) -> Result<&mut AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[u8], Output=[u8]>
    {
        let (ptr, len) = if let Some(slice) = self.as_bytes().get(range) {
            if !slice.is_ascii() {
                let offset = slice.as_ptr() as usize - self.as_ptr() as usize;
                let not_ascii = slice.iter().take_while(|&b| b.is_ascii()).count();
                return Err(AsAsciiStrError(offset+not_ascii));
            }
            (slice.as_ptr(), slice.len())
        } else {
            return Err(AsAsciiStrError(self.len()));
        };
        unsafe {
            let ptr = ptr as *const AsciiChar as *mut AsciiChar;
            let slice = core::slice::from_raw_parts_mut(ptr, len);
            Ok(slice.into())
        }
    }
    fn as_mut_ascii_str(&mut self) -> Result<&mut AsciiStr, AsAsciiStrError> {
        match self.bytes().position(|b| b > 127) {
            Some(index) => Err(AsAsciiStrError(index)),
            None => unsafe { Ok(self.as_mut_ascii_str_unchecked()) },
        }
    }
    #[inline]
    unsafe fn as_mut_ascii_str_unchecked(&mut self) -> &mut AsciiStr {
        let ptr = self as *mut str as *mut AsciiStr;
        &mut *ptr
    }
}

/// Note that the trailing null byte will be removed in the conversion.
#[cfg(feature = "std")]
impl AsAsciiStr for CStr {
    type Inner = u8;
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
        where R: SliceIndex<[u8], Output=[u8]>
    {
        self.to_bytes().slice_ascii(range)
    }
    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        self.to_bytes().as_ascii_str()
    }
    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        self.to_bytes().as_ascii_str_unchecked()
    }
}

#[cfg(test)]
mod tests {
    use AsciiChar;
    use super::{AsciiStr, AsAsciiStr, AsMutAsciiStr, AsAsciiStrError};

    #[test]
    fn generic_as_ascii_str() {
        fn generic<C: AsAsciiStr + ?Sized>(c: &C) -> Result<&AsciiStr, AsAsciiStrError> {
            c.as_ascii_str()
        }
        let arr = [AsciiChar::A];
        let ascii_str: &AsciiStr = arr.as_ref().into();
        assert_eq!(generic("A"), Ok(ascii_str));
        assert_eq!(generic(&b"A"[..]), Ok(ascii_str));
        assert_eq!(generic(ascii_str), Ok(ascii_str));
        assert_eq!(generic(&"A"), Ok(ascii_str));
        assert_eq!(generic(&ascii_str), Ok(ascii_str));
        assert_eq!(generic(&mut "A"), Ok(ascii_str));
    }

    #[cfg(feature = "std")]
    #[test]
    fn cstring_as_ascii_str() {
        use std::ffi::CString;
        fn generic<C: AsAsciiStr + ?Sized>(c: &C) -> Result<&AsciiStr, AsAsciiStrError> {
            c.as_ascii_str()
        }
        let arr = [AsciiChar::A];
        let ascii_str: &AsciiStr = arr.as_ref().into();
        let cstr = CString::new("A").unwrap();
        assert_eq!(generic(&*cstr), Ok(ascii_str));
    }

    #[test]
    fn generic_as_mut_ascii_str() {
        fn generic_mut<C: AsMutAsciiStr + ?Sized>(
            c: &mut C,
        ) -> Result<&mut AsciiStr, AsAsciiStrError> {
            c.as_mut_ascii_str()
        }

        let mut arr_mut = [AsciiChar::B];
        let mut ascii_str_mut: &mut AsciiStr = arr_mut.as_mut().into();
        // Need a second reference to prevent overlapping mutable borrows
        let mut arr_mut_2 = [AsciiChar::B];
        let ascii_str_mut_2: &mut AsciiStr = arr_mut_2.as_mut().into();
        assert_eq!(generic_mut(&mut ascii_str_mut), Ok(&mut *ascii_str_mut_2));
        assert_eq!(generic_mut(ascii_str_mut), Ok(&mut *ascii_str_mut_2));
    }

    #[test]
    fn as_ascii_str() {
        macro_rules! err {{$i:expr} => {Err(AsAsciiStrError($i))}}
        let s = "abčd";
        let b = s.as_bytes();
        assert_eq!(s.as_ascii_str(), err!(2));
        assert_eq!(b.as_ascii_str(), err!(2));
        let a: &AsciiStr = [AsciiChar::a, AsciiChar::b][..].as_ref();
        assert_eq!(s[..2].as_ascii_str(), Ok(a));
        assert_eq!(b[..2].as_ascii_str(), Ok(a));
        assert_eq!(s.slice_ascii(..2), Ok(a));
        assert_eq!(b.slice_ascii(..2), Ok(a));
        assert_eq!(s.slice_ascii(..=2), err!(2));
        assert_eq!(b.slice_ascii(..=2), err!(2));
        assert_eq!(s.get_ascii(4), Some(AsciiChar::d));
        assert_eq!(b.get_ascii(4), Some(AsciiChar::d));
        assert_eq!(s.get_ascii(3), None);
        assert_eq!(b.get_ascii(3), None);
        assert_eq!(s.get_ascii(b.len()), None);
        assert_eq!(b.get_ascii(b.len()), None);
        assert_eq!(a.get_ascii(0), Some(AsciiChar::a));
        assert_eq!(a.get_ascii(a.len()), None);
    }

    #[test]
    #[cfg(feature = "std")]
    fn cstr_as_ascii_str() {
        use std::ffi::CStr;
        macro_rules! err {{$i:expr} => {Err(AsAsciiStrError($i))}}
        let cstr = CStr::from_bytes_with_nul(b"a\xbbcde\xffg\0").unwrap();
        assert_eq!(cstr.as_ascii_str(), err!(1));
        assert_eq!(cstr.slice_ascii(2..), err!(5));
        assert_eq!(cstr.get_ascii(5), None);
        assert_eq!(cstr.get_ascii(6), Some(AsciiChar::g));
        assert_eq!(cstr.get_ascii(7), None);
        let aslice = &[AsciiChar::X, AsciiChar::Y, AsciiChar::Z, AsciiChar::Null][..];
        let astr: &AsciiStr = aslice.as_ref();
        let cstr = CStr::from_bytes_with_nul(astr.as_bytes()).unwrap();
        assert_eq!(cstr.slice_ascii(..2), Ok(&astr[..2]));
        assert_eq!(cstr.as_ascii_str(), Ok(&astr[..3]));
    }

    #[test]
    #[cfg(feature = "std")]
    fn as_mut_ascii_str() {
        macro_rules! err {{$i:expr} => {Err(AsAsciiStrError($i))}}
        let mut s: String = "abčd".to_string();
        let mut b: Vec<u8> = s.clone().into();
        let mut first = [AsciiChar::a, AsciiChar::b];
        let mut second = [AsciiChar::d];
        assert_eq!(s.as_mut_ascii_str(), err!(2));
        assert_eq!(b.as_mut_ascii_str(), err!(2));
        assert_eq!(s.slice_ascii_mut(..), err!(2));
        assert_eq!(b.slice_ascii_mut(..), err!(2));
        assert_eq!(s[..2].as_mut_ascii_str(), Ok((&mut first[..]).into()));
        assert_eq!(b[..2].as_mut_ascii_str(), Ok((&mut first[..]).into()));
        assert_eq!(s.slice_ascii_mut(0..2), Ok((&mut first[..]).into()));
        assert_eq!(b.slice_ascii_mut(0..2), Ok((&mut first[..]).into()));
        assert_eq!(s.slice_ascii_mut(4..), Ok((&mut second[..]).into()));
        assert_eq!(b.slice_ascii_mut(4..), Ok((&mut second[..]).into()));
        assert_eq!(s.slice_ascii_mut(4..=10), err!(5));
        assert_eq!(b.slice_ascii_mut(4..=10), err!(5));
    }

    #[test]
    fn default() {
        let default: &'static AsciiStr = Default::default();
        assert!(default.is_empty());
    }

    #[test]
    fn index() {
        let mut arr = [AsciiChar::A, AsciiChar::B, AsciiChar::C, AsciiChar::D];
        {
            let a: &AsciiStr = arr[..].into();
            assert_eq!(a[..].as_slice(), &a.as_slice()[..]);
            assert_eq!(a[..4].as_slice(), &a.as_slice()[..4]);
            assert_eq!(a[4..].as_slice(), &a.as_slice()[4..]);
            assert_eq!(a[2..3].as_slice(), &a.as_slice()[2..3]);
            assert_eq!(a[..=3].as_slice(), &a.as_slice()[..=3]);
            assert_eq!(a[1..=1].as_slice(), &a.as_slice()[1..=1]);
        }
        let mut copy = arr;
        let a_mut: &mut AsciiStr = {&mut arr[..]}.into();
        assert_eq!(a_mut[..].as_mut_slice(), &mut copy[..]);
        assert_eq!(a_mut[..2].as_mut_slice(), &mut copy[..2]);
        assert_eq!(a_mut[3..].as_mut_slice(), &mut copy[3..]);
        assert_eq!(a_mut[4..4].as_mut_slice(), &mut copy[4..4]);
        assert_eq!(a_mut[..=0].as_mut_slice(), &mut copy[..=0]);
        assert_eq!(a_mut[0..=2].as_mut_slice(), &mut copy[0..=2]);
    }

    #[test]
    fn as_str() {
        let b = b"( ;";
        let v = AsciiStr::from_ascii(b).unwrap();
        assert_eq!(v.as_str(), "( ;");
        assert_eq!(AsRef::<str>::as_ref(v), "( ;");
    }

    #[test]
    fn as_bytes() {
        let b = b"( ;";
        let v = AsciiStr::from_ascii(b).unwrap();
        assert_eq!(v.as_bytes(), b"( ;");
        assert_eq!(AsRef::<[u8]>::as_ref(v), b"( ;");
    }

    #[test]
    fn make_ascii_case() {
        let mut bytes = ([b'a', b'@', b'A'], [b'A', b'@', b'a']);
        let a = bytes.0.as_mut_ascii_str().unwrap();
        let b = bytes.1.as_mut_ascii_str().unwrap();
        assert!(a.eq_ignore_ascii_case(b));
        assert!(b.eq_ignore_ascii_case(a));
        a.make_ascii_lowercase();
        b.make_ascii_uppercase();
        assert_eq!(a, "a@a");
        assert_eq!(b, "A@A");
    }

    #[test]
    #[cfg(feature = "std")]
    fn to_ascii_case() {
        let bytes = ([b'a', b'@', b'A'], [b'A', b'@', b'a']);
        let a = bytes.0.as_ascii_str().unwrap();
        let b = bytes.1.as_ascii_str().unwrap();
        assert_eq!(a.to_ascii_lowercase().as_str(), "a@a");
        assert_eq!(a.to_ascii_uppercase().as_str(), "A@A");
        assert_eq!(b.to_ascii_lowercase().as_str(), "a@a");
        assert_eq!(b.to_ascii_uppercase().as_str(), "A@A");
    }

    #[test]
    fn chars_iter() {
        let chars = &[b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', b'\0'];
        let ascii = AsciiStr::from_ascii(chars).unwrap();
        for (achar, byte) in ascii.chars().zip(chars.iter().cloned()) {
            assert_eq!(achar, byte);
        }
    }

    #[test]
    fn chars_iter_mut() {
        let chars = &mut [b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', b'\0'];
        let ascii = chars.as_mut_ascii_str().unwrap();
        *ascii.chars_mut().next().unwrap() = AsciiChar::H;
        assert_eq!(ascii[0], b'H');
    }

    #[test]
    fn lines_iter() {
        use core::iter::Iterator;

        let lines: [&str; 4] = ["foo", "bar", "", "baz"];
        let joined = "foo\r\nbar\n\nbaz\n";
        let ascii = AsciiStr::from_ascii(joined.as_bytes()).unwrap();
        for (asciiline, line) in ascii.lines().zip(&lines) {
            assert_eq!(asciiline, *line);
        }
        assert_eq!(ascii.lines().count(), lines.len());

        let lines: [&str; 4] = ["foo", "bar", "", "baz"];
        let joined = "foo\r\nbar\n\nbaz";
        let ascii = AsciiStr::from_ascii(joined.as_bytes()).unwrap();
        for (asciiline, line) in ascii.lines().zip(&lines) {
            assert_eq!(asciiline, *line);
        }
        assert_eq!(ascii.lines().count(), lines.len());

        let trailing_line_break = b"\n";
        let ascii = AsciiStr::from_ascii(&trailing_line_break).unwrap();
        let mut line_iter = ascii.lines();
        assert_eq!(line_iter.next(), Some(AsciiStr::from_ascii("").unwrap()));
        assert_eq!(line_iter.next(), None);

        let empty_lines = b"\n\r\n\n\r\n";
        let mut iter_count = 0;
        let ascii = AsciiStr::from_ascii(&empty_lines).unwrap();
        for line in ascii.lines() {
            iter_count += 1;
            assert!(line.is_empty());
        }
        assert_eq!(4, iter_count);
    }

    #[test]
    fn lines_iter_rev() {
        let joined = "foo\r\nbar\n\nbaz\n";
        let ascii = AsciiStr::from_ascii(joined.as_bytes()).unwrap();
        assert_eq!(ascii.lines().rev().count(), 4);
        assert_eq!(ascii.lines().rev().count(), joined.lines().rev().count());
        for (asciiline, line) in ascii.lines().rev().zip(joined.lines().rev()) {
            assert_eq!(asciiline, line);
        }
        let mut iter = ascii.lines();
        assert_eq!(iter.next(), Some("foo".as_ascii_str().unwrap()));
        assert_eq!(iter.next_back(), Some("baz".as_ascii_str().unwrap()));
        assert_eq!(iter.next_back(), Some("".as_ascii_str().unwrap()));
        assert_eq!(iter.next(), Some("bar".as_ascii_str().unwrap()));
    }

    #[test]
    fn lines_iter_empty() {
        assert_eq!("".as_ascii_str().unwrap().lines().next(), None);
        assert_eq!("".as_ascii_str().unwrap().lines().next_back(), None);
        assert_eq!("".lines().next(), None);
    }

    #[test]
    fn split_str() {
        fn split_equals_str(haystack: &str, needle: char) {
            let mut strs = haystack.split(needle);
            let mut asciis = haystack.as_ascii_str().unwrap()
                .split(AsciiChar::from_ascii(needle).unwrap())
                .map(|a| a.as_str());
            loop {
                assert_eq!(asciis.size_hint(), strs.size_hint());
                let (a, s) = (asciis.next(), strs.next());
                assert_eq!(a, s);
                if a == None {
                    break;
                }
            }
            // test fusedness if str's version is fused
            if strs.next() == None {
                assert_eq!(asciis.next(), None);
            }
        }
        split_equals_str("", '=');
        split_equals_str("1,2,3", ',');
        split_equals_str("foo;bar;baz;", ';');
        split_equals_str("|||", '|');
        split_equals_str(" a  b  c ", ' ');
    }

    #[test]
    fn split_str_rev() {
        let words = " foo  bar baz ";
        let ascii = words.as_ascii_str().unwrap();
        for (word, asciiword) in words.split(' ').rev().zip(ascii.split(AsciiChar::Space).rev()) {
            assert_eq!(asciiword, word);
        }
        let mut iter = ascii.split(AsciiChar::Space);
        assert_eq!(iter.next(), Some("".as_ascii_str().unwrap()));
        assert_eq!(iter.next_back(), Some("".as_ascii_str().unwrap()));
        assert_eq!(iter.next(), Some("foo".as_ascii_str().unwrap()));
        assert_eq!(iter.next_back(), Some("baz".as_ascii_str().unwrap()));
        assert_eq!(iter.next_back(), Some("bar".as_ascii_str().unwrap()));
        assert_eq!(iter.next(), Some("".as_ascii_str().unwrap()));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn split_str_empty() {
        let empty = <&AsciiStr>::default();
        let mut iter = empty.split(AsciiChar::NAK);
        assert_eq!(iter.next(), Some(empty));
        assert_eq!(iter.next(), None);
        let mut iter = empty.split(AsciiChar::NAK);
        assert_eq!(iter.next_back(), Some(empty));
        assert_eq!(iter.next_back(), None);
        assert_eq!("".split('s').next(), Some("")); // str.split() also produces one element
    }

    #[test]
    #[cfg(feature = "std")]
    fn fmt_ascii_str() {
        let s = "abc".as_ascii_str().unwrap();
        assert_eq!(format!("{}", s), "abc".to_string());
        assert_eq!(format!("{:?}", s), "\"abc\"".to_string());
    }
}
