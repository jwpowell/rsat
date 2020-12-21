use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, MulAssign,
    Neg, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use std::convert::{TryFrom, TryInto};

use crate::bits::{Bit, Bits};

pub struct Word {
    bits: Bits,
    ids: Vec<u32>,
}

impl Clone for Word {
    fn clone(&self) -> Word {
        let word = Word {
            bits: self.bits.clone(),
            ids: self.ids.clone(),
        };

        for id in &word.ids {
            self.bits.incr(*id);
        }

        word
    }
}

impl Drop for Word {
    fn drop(&mut self) {
        for id in &self.ids {
            self.bits.decr(*id);
        }
    }
}

impl Word {
    pub fn var(bits: &Bits, width: usize) -> Word {
        let mut ids = Vec::with_capacity(width);

        for _ in 0..width {
            let id = bits.var();
            bits.incr(id);

            ids.push(id);
        }

        Word {
            ids,
            bits: bits.clone(),
        }
    }

    pub fn width(&self) -> usize {
        self.ids.len()
    }

    pub fn from_u64(bits: &Bits, width: usize, val: u64) -> Word {
        let mut ids = Vec::with_capacity(width);

        for i in 0..width {
            let id = bits.val((val >> i) & 1 != 0);
            bits.incr(id);
            ids.push(id);
        }

        Word {
            ids,
            bits: bits.clone(),
        }
    }

    pub fn slice(&self, lo: usize, hi: usize) -> Word {
        let word = Word {
            bits: self.bits.clone(),
            ids: self.ids[lo..=hi].to_vec(),
        };

        for id in &word.ids {
            self.bits.incr(*id);
        }

        word
    }

    pub fn concat(&self, rhs: &Word) -> Word {
        assert!(self.bits.ptr_eq(&rhs.bits));

        let word = Word {
            ids: rhs.ids.iter().chain(self.ids.iter()).copied().collect(),
            bits: self.bits.clone(),
        };

        for id in &word.ids {
            self.bits.incr(*id);
        }

        word
    }
}

impl BitAnd<&Word> for &Word {
    type Output = Word;

    fn bitand(self, rhs: &Word) -> Self::Output {
        assert!(self.bits.ptr_eq(&rhs.bits));
        assert_eq!(self.width(), rhs.width());

        Word {
            bits: self.bits.clone(),

            ids: self
                .ids
                .iter()
                .zip(rhs.ids.iter())
                .map(|(a, b)| {
                    if self.bits.is_false(*a) || self.bits.is_true(*b) {
                        self.bits.incr(*a);
                        return *a;
                    }

                    if self.bits.is_true(*a) || self.bits.is_false(*b) {
                        self.bits.incr(*b);
                        return *b;
                    }

                    if a == b {
                        self.bits.incr(*a);
                        return *a;
                    }

                    let c = self.bits.and(*a, *b);

                    self.bits.incr(*a);
                    self.bits.incr(*b);
                    self.bits.incr(c);

                    c
                })
                .collect(),
        }
    }
}

impl BitAndAssign<&Word> for Word {
    fn bitand_assign(&mut self, rhs: &Word) {
        let c = self.bitand(rhs);
        *self = c;
    }
}

impl BitOr<&Word> for &Word {
    type Output = Word;

    fn bitor(self, rhs: &Word) -> Self::Output {
        assert!(self.bits.ptr_eq(&rhs.bits));
        assert_eq!(self.width(), rhs.width());

        Word {
            bits: self.bits.clone(),

            ids: self
                .ids
                .iter()
                .zip(rhs.ids.iter())
                .map(|(a, b)| {
                    if self.bits.is_false(*a) || self.bits.is_true(*b) {
                        self.bits.incr(*b);
                        return *b;
                    }

                    if self.bits.is_true(*a) || self.bits.is_false(*b) {
                        self.bits.incr(*a);
                        return *a;
                    }

                    if a == b {
                        self.bits.incr(*a);
                        return *a;
                    }

                    let c = self.bits.or(*a, *b);

                    self.bits.incr(*a);
                    self.bits.incr(*b);
                    self.bits.incr(c);

                    c
                })
                .collect(),
        }
    }
}

impl BitOrAssign<&Word> for Word {
    fn bitor_assign(&mut self, rhs: &Word) {
        let c = self.bitor(rhs);
        *self = c;
    }
}

impl Not for &Word {
    type Output = Word;

    fn not(self) -> Word {
        Word {
            bits: self.bits.clone(),
            ids: self
                .ids
                .iter()
                .map(|a| {
                    if self.bits.is_false(*a) {
                        let c = self.bits.val(true);
                        self.bits.incr(c);

                        c
                    } else if self.bits.is_true(*a) {
                        let c = self.bits.val(false);
                        self.bits.incr(c);

                        c
                    } else {
                        let c = self.bits.not(*a);

                        self.bits.incr(*a);
                        self.bits.incr(c);

                        c
                    }
                })
                .collect(),
        }
    }
}

impl BitXor<&Word> for &Word {
    type Output = Word;
    fn bitxor(self, rhs: &Word) -> Self::Output {
        let t1 = !self;
        let t2 = !rhs;

        let t3 = &t1 & rhs;
        let t4 = self & &t2;

        &t3 | &t4
    }
}

impl BitXorAssign<&Word> for Word {
    fn bitxor_assign(&mut self, rhs: &Word) {
        let c = self.bitxor(rhs);
        *self = c;
    }
}

impl TryFrom<&Word> for u64 {
    type Error = ();

    fn try_from(value: &Word) -> Result<u64, Self::Error> {
        let mut n = 0;
        for (i, id) in value.ids.iter().enumerate() {
            if value.bits.is_true(*id) {
                n |= 1u64 << i;
            } else if !value.bits.is_false(*id) {
                return Err(());
            }
        }

        Ok(n)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn convert_01() {
        let bits = Bits::new();

        for k in 0u64..=255 {
            let a = Word::from_u64(&bits, 8, k);
            let j = u64::try_from(&a).unwrap();

            assert_eq!(k, j);
        }
    }

    #[test]
    fn and_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for j in 0u8..=255 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let b = Word::from_u64(&bits, 8, j as u64);
                let c = &a & &b;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k & j);
            }
        }
    }

    #[test]
    fn and_02() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for j in 0u8..=255 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let b = Word::from_u64(&bits, 8, j as u64);
                let mut c = a.clone();
                c &= &b;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k & j);
            }
        }
    }

    #[test]
    fn or_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for j in 0u8..=255 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let b = Word::from_u64(&bits, 8, j as u64);
                let c = &a | &b;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k | j);
            }
        }
    }

    #[test]
    fn or_02() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for j in 0u8..=255 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let b = Word::from_u64(&bits, 8, j as u64);
                let mut c = a.clone();
                c |= &b;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k | j);
            }
        }
    }

    #[test]
    fn xor_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for j in 0u8..=255 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let b = Word::from_u64(&bits, 8, j as u64);
                let c = &a ^ &b;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k ^ j);
            }
        }
    }

    #[test]
    fn xor_02() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for j in 0u8..=255 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let b = Word::from_u64(&bits, 8, j as u64);
                let mut c = a.clone();
                c ^= &b;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k ^ j);
            }
        }
    }

    #[test]
    fn not_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            let a = Word::from_u64(&bits, 8, k as u64);
            let c = !&a;

            let l = u64::try_from(&c).unwrap() as u8;

            assert_eq!(l, !k);
        }
    }
}
