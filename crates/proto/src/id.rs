use std::fmt;
use std::str::from_utf8;

use crate::NonCryptographicRng;

// deliberate subset of ice-char, etc that are "safe"
const CHARS: &[u8] = b"abcdefghijklmnopqrstuvxyzABCDEFGHIJKLMNOPQRSTUVXYZ0123456789";
const BASE62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct Id<const L: usize>([u8; L]);

impl<const L: usize> Id<L> {
    pub fn random() -> Id<L> {
        let mut x = [0; L];
        for val in x.iter_mut().take(L) {
            let y: f32 = NonCryptographicRng::f32();
            let idx = (CHARS.len() as f32 * y).floor() as usize;
            *val = CHARS[idx];
        }
        Id(x)
    }

    /// Starts a counter whose initial base-62 value fits in `L` characters.
    pub fn random_counter_start() -> u64 {
        const {
            assert!(L > 0);
        }
        let value = NonCryptographicRng::u64();
        let range = (0..L).try_fold(1_u64, |n, _| n.checked_mul(BASE62.len() as u64));
        range.map_or(value, |range| value % range)
    }

    /// Encodes the counter as base-62 and advances it with wrapping addition.
    ///
    /// The array must fit every `u64` value, requiring at least 11 bytes.
    ///
    /// ```compile_fail
    /// let mut counter = 0;
    /// str0m_proto::Id::<10>::next(&mut counter);
    /// ```
    pub fn next(counter: &mut u64) -> Self {
        const {
            assert!(L > u64::MAX.ilog(BASE62.len() as u64) as usize);
        }
        let mut value = *counter;
        *counter = counter.wrapping_add(1);
        let mut bytes = [b' '; L];
        let mut len = 0;
        loop {
            bytes[len] = BASE62[(value % BASE62.len() as u64) as usize];
            len += 1;
            value /= BASE62.len() as u64;
            if value == 0 {
                break;
            }
        }
        bytes[..len].reverse();
        Id(bytes)
    }

    pub fn into_array(self) -> [u8; L] {
        self.0
    }
}

impl<const L: usize> Default for Id<L> {
    fn default() -> Self {
        Id::random()
    }
}

impl<const L: usize> fmt::Display for Id<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = from_utf8(&self.0).expect("ascii characters");
        write!(f, "{}", s.trim_end())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn counter_encoding() {
        for (value, expected) in [
            (0, "0"),
            (9, "9"),
            (10, "a"),
            (35, "z"),
            (36, "A"),
            (61, "Z"),
            (62, "10"),
            (62_u64.pow(3) - 1, "ZZZ"),
            (62_u64.pow(3), "1000"),
            (u64::MAX, "lYGhA16ahyf"),
        ] {
            let mut counter = value;
            let id = Id::<11>::next(&mut counter);
            assert_eq!(id.to_string(), expected);
            assert_eq!(counter, value.wrapping_add(1));
            let bytes = id.into_array();
            assert_eq!(&bytes[..expected.len()], expected.as_bytes());
            assert!(bytes[expected.len()..].iter().all(|b| *b == b' '));
        }
    }

    #[test]
    fn counter_start_preserves_seeded_range() {
        for seed in [0, 1, 42, 43, u64::MAX] {
            fastrand::seed(seed);
            let value = NonCryptographicRng::u64();
            fastrand::seed(seed);
            assert_eq!(Id::<3>::random_counter_start(), value % 62_u64.pow(3));
            fastrand::seed(seed);
            assert_eq!(Id::<20>::random_counter_start(), value);
        }
    }

    #[test]
    fn random_id_keeps_full_width() {
        let id = Id::<20>::random();
        assert_eq!(id.to_string().len(), 20);
        assert!(id.into_array().iter().all(|b| CHARS.contains(b)));
    }
}
