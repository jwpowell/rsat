use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Mul,
    MulAssign, Neg, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
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
            ids.push(bits.var());
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

    pub fn rotr(&self, k: usize) -> Word {
        let k = k % self.width();

        let word = Word {
            bits: self.bits.clone(),
            ids: self
                .ids
                .iter()
                .cycle()
                .skip(k)
                .take(self.width())
                .copied()
                .collect(),
        };

        for id in &word.ids {
            self.bits.incr(*id);
        }

        word
    }

    pub fn rotl(&self, k: usize) -> Word {
        let k = k % self.width();

        self.rotr(self.width() - k)
    }

    pub fn cond(test: &Word, yes: &Word, no: &Word) -> Word {
        assert!(test.bits.ptr_eq(&yes.bits));
        assert!(test.bits.ptr_eq(&no.bits));
        assert_eq!(test.width(), 1);
        assert_eq!(yes.width(), no.width());

        let bits = &test.bits;

        let t = test.ids[0];

        Word {
            bits: bits.clone(),
            ids: yes
                .ids
                .iter()
                .zip(no.ids.iter())
                .map(|(y, n)| bits.cond(t, *y, *n))
                .collect(),
        }
    }

    fn fold<F>(&self, init: u32, mut f: F) -> Word
    where
        F: FnMut(&Bits, u32, u32) -> u32,
    {
        let mut value = init;

        self.bits.incr(value);

        for id in &self.ids {
            let old = value;
            value = f(&self.bits, value, *id);
            self.bits.decr(old);
        }

        Word {
            bits: self.bits.clone(),
            ids: vec![value],
        }
    }

    pub fn all(&self) -> Word {
        let init = self.bits.val(false);
        let word = self.fold(init, |bits, a, b| bits.and(a, b));

        self.bits.decr(init);

        word
    }

    pub fn any(&self) -> Word {
        let init = self.bits.val(true);
        let word = self.fold(init, |bits, a, b| bits.or(a, b));

        self.bits.decr(init);

        word
    }

    pub fn parity(&self) -> Word {
        let init = self.bits.val(false);
        let word = self.fold(init, |bits, a, b| bits.xor(a, b));

        self.bits.decr(init);

        word
    }

    pub fn less_than(&self, rhs: &Word) -> Word {
        todo!()
        //self.ids.iter().rev().zip(rhs.ids.iter().rev()).
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
                .map(|(a, b)| self.bits.and(*a, *b))
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
                .map(|(a, b)| self.bits.or(*a, *b))
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
            ids: self.ids.iter().map(|a| self.bits.not(*a)).collect(),
        }
    }
}

impl BitXor<&Word> for &Word {
    type Output = Word;
    fn bitxor(self, rhs: &Word) -> Self::Output {
        assert!(self.bits.ptr_eq(&rhs.bits));
        assert_eq!(self.width(), rhs.width());

        Word {
            bits: self.bits.clone(),

            ids: self
                .ids
                .iter()
                .zip(rhs.ids.iter())
                .map(|(a, b)| self.bits.xor(*a, *b))
                .collect(),
        }
    }
}

impl BitXorAssign<&Word> for Word {
    fn bitxor_assign(&mut self, rhs: &Word) {
        let c = self.bitxor(rhs);
        *self = c;
    }
}

impl Shr<usize> for &Word {
    type Output = Word;

    fn shr(self, rhs: usize) -> Self::Output {
        Word {
            bits: self.bits.clone(),
            ids: self
                .ids
                .iter()
                .skip(rhs)
                .copied()
                .map(|a| {
                    self.bits.incr(a);
                    a
                })
                .chain(std::iter::repeat_with(|| self.bits.val(false)))
                .take(self.width())
                .collect(),
        }
    }
}

impl Shl<usize> for &Word {
    type Output = Word;

    fn shl(self, rhs: usize) -> Self::Output {
        Word {
            bits: self.bits.clone(),
            ids: std::iter::repeat_with(|| self.bits.val(false))
                .take(rhs)
                .chain(self.ids.iter().copied().map(|a| {
                    self.bits.incr(a);
                    a
                }))
                .take(self.width())
                .collect(),
        }
    }
}

impl Add<&Word> for &Word {
    type Output = Word;

    fn add(self, rhs: &Word) -> Word {
        assert!(self.bits.ptr_eq(&rhs.bits));
        assert_eq!(self.width(), rhs.width());

        let mut carry = self.bits.val(false);

        let mut ids = Vec::with_capacity(self.width());

        for (x, y) in self.ids.iter().zip(rhs.ids.iter()) {
            let (s, c) = self.bits.full_adder(*x, *y, carry);

            ids.push(s);

            self.bits.decr(carry);
            carry = c;
        }

        self.bits.decr(carry);

        Word {
            bits: self.bits.clone(),
            ids,
        }
    }
}

impl AddAssign<&Word> for Word {
    fn add_assign(&mut self, rhs: &Word) {
        let c = &*self + rhs;

        *self = c;
    }
}

impl Neg for &Word {
    type Output = Word;

    fn neg(self) -> Self::Output {
        self.not().add(&Word::from_u64(&self.bits, self.width(), 1))
    }
}

impl Sub<&Word> for &Word {
    type Output = Word;

    fn sub(self, rhs: &Word) -> Self::Output {
        self + &-rhs
    }
}

impl SubAssign<&Word> for Word {
    fn sub_assign(&mut self, rhs: &Word) {
        let c = &*self - rhs;

        *self = c;
    }
}

impl Mul<&Word> for &Word {
    type Output = Word;

    fn mul(self, rhs: &Word) -> Self::Output {
        assert!(self.bits.ptr_eq(&rhs.bits));
        assert_eq!(self.width(), rhs.width());

        let zero = Word::from_u64(&self.bits, self.width(), 0);
        let mut product = zero.clone();

        for k in 0..self.width() {
            product += &(&Word::cond(&rhs.slice(k, k), self, &zero) << k);
        }

        product
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

    const MAX: u64 = 15;
    const BITS: usize = 4;

    fn total_refcounts(bits: &Bits) -> u32 {
        let mut sum = 0;

        for id in 0..bits.size() {
            sum += bits.refcount(id as u32);
        }

        sum
    }

    #[test]
    fn convert_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            let a = Word::from_u64(&bits, BITS, k);
            let j = u64::try_from(&a).unwrap();

            assert_eq!(k, j);
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn and_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let c = &a & &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k & j);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn and_02() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let mut c = a.clone();
                c &= &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k & j);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn or_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let c = &a | &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k | j);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn or_02() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let mut c = a.clone();
                c |= &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k | j);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn xor_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let c = &a ^ &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k ^ j);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn xor_02() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let mut c = a.clone();
                c ^= &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k ^ j);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn not_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            let a = Word::from_u64(&bits, BITS, k);
            let c = !&a;

            let l = u64::try_from(&c).unwrap();

            assert_eq!(l & MAX, !k & MAX);
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn shr_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for n in 0..8 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let c = &a >> n;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k >> n);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn shl_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for n in 0..8 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let c = &a << n;

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k << n);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn rotr_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for n in 0..8 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let c = a.rotr(n);

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k.rotate_right(n as u32));
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn rotl_01() {
        let bits = Bits::new();

        for k in 0u8..=255 {
            for n in 0..8 {
                let a = Word::from_u64(&bits, 8, k as u64);
                let c = a.rotl(n);

                let l = u64::try_from(&c).unwrap() as u8;

                assert_eq!(l, k.rotate_left(n as u32));
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn add_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let c = &a + &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k.wrapping_add(j) & MAX);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn add_02() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let mut c = a.clone();
                c += &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k.wrapping_add(j) & MAX);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn neg_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            let a = Word::from_u64(&bits, BITS, k);
            let c = -&a;

            let l = u64::try_from(&c).unwrap();

            assert_eq!(l & MAX, k.wrapping_neg() & MAX);
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn sub_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let c = &a - &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k.wrapping_sub(j) & MAX);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn sub_02() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let mut c = a.clone();
                c -= &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k.wrapping_sub(j) & MAX);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]
    fn mul_01() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let c = &a * &b;

                let l = u64::try_from(&c).unwrap();

                assert_eq!(l, k.wrapping_mul(j) & MAX);
            }
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }

    #[test]

    fn var_mul_01() {
        let mut bits = Bits::new();

        {
            let a = Word::var(&bits, 64);
            let b = Word::var(&bits, 64);
            let c = &a * &b;
        }

        assert_eq!(total_refcounts(&bits), 0, "refcount expected to be zero");
    }
}
