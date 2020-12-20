use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, MulAssign,
    Neg, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use std::convert::TryFrom;
use std::fmt;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Bit {
    Unused,
    Var,
    Val(bool),

    And(u32, u32),
    Or(u32, u32),
    Not(u32),
}

struct _Words {
    bits: Vec<(u32, Bit)>,
    garbage: Vec<u32>,
}

pub struct Words(Rc<RefCell<_Words>>);

impl Default for Words {
    fn default() -> Words {
        Words::new()
    }
}

impl Words {
    pub fn new() -> Words {
        Words(Rc::new(RefCell::new(_Words {
            bits: vec![],
            garbage: vec![],
        })))
    }

    fn alloc_bit(&self, bit: Bit) -> u32 {
        let mut inner = self.0.borrow_mut();

        if let Some(id) = inner.garbage.pop() {
            inner.bits[id as usize] = (1, bit);
            return id;
        }

        let id = inner.bits.len() as u32;
        inner.bits.push((1, bit));

        id
    }

    fn incr(&self, id: u32) {
        let mut inner = self.0.borrow_mut();
        inner.bits[id as usize].0 += 1;
    }

    fn decr(&self, id: u32) {
        let mut inner = self.0.borrow_mut();
        let mut pending = vec![id];

        while let Some(id) = pending.pop() {
            let refcount = &mut inner.bits[id as usize].0;

            assert!(*refcount > 0);
            *refcount -= 1;

            if *refcount == 0 {
                inner.garbage.push(id);

                match inner.bits[id as usize].1 {
                    Bit::Unused => unreachable!(),

                    Bit::Var => {}

                    Bit::Val(..) => {}

                    Bit::And(l, r) => {
                        pending.push(l);
                        pending.push(r);
                    }

                    Bit::Or(l, r) => {
                        pending.push(l);
                        pending.push(r);
                    }

                    Bit::Not(e) => pending.push(e),
                }
            }
        }
    }

    fn is_val(&self, id: u32, v: bool) -> bool {
        matches!(self.0.borrow().bits[id as usize].1,
            Bit::Val(w) if w == v)
    }

    fn is_false(&self, id: u32) -> bool {
        self.is_val(id, false)
    }

    fn is_true(&self, id: u32) -> bool {
        self.is_val(id, true)
    }

    pub fn var(&self, width: usize) -> Word {
        let mut word = Word::alloc(self, width);
        let mut inner = self.0.borrow_mut();

        for bit in &word.bits {
            inner.bits[*bit as usize] = (1, Bit::Var);
        }

        word
    }

    pub fn from_u64(&self, width: usize, n: u64) -> Word {
        let mut n = n;

        let mut bits = Vec::with_capacity(width);

        for _ in 0..width {
            if n % 2 == 0 {
                bits.push(self.alloc_bit(Bit::Val(false)));
            } else {
                bits.push(self.alloc_bit(Bit::Val(true)));
            }

            n /= 2;
        }

        let word = Word {
            words: self.clone(),
            bits,
        };

        word.incr();

        word
    }
}

impl Clone for Words {
    fn clone(&self) -> Words {
        Words(Rc::clone(&self.0))
    }
}

pub struct Word {
    words: Words,
    bits: Vec<u32>,
}

impl Clone for Word {
    fn clone(&self) -> Word {
        let word = Word {
            words: self.words.clone(),
            bits: self.bits.clone(),
        };

        word.incr();

        word
    }
}

impl Drop for Word {
    fn drop(&mut self) {
        self.decr();
    }
}

impl Word {
    fn alloc(words: &Words, width: usize) -> Word {
        let mut bits = Vec::with_capacity(width);

        for _ in 0..width {
            bits.push(words.alloc_bit(Bit::Unused));
        }

        Word {
            words: words.clone(),
            bits,
        }
    }

    fn incr(&self) {
        for bit in &self.bits {
            self.words.incr(*bit);
        }
    }

    fn decr(&self) {
        for bit in &self.bits {
            self.words.decr(*bit);
        }
    }

    pub fn width(&self) -> usize {
        self.bits.len()
    }

    pub fn simplify(&mut self) {
        todo!()
    }

    pub fn concat(&self, other: &Word) -> Word {
        assert!(Rc::ptr_eq(&self.words.0, &other.words.0));

        let mut bits = Vec::with_capacity(self.width() + other.width());

        bits.extend(other.bits.iter().chain(self.bits.iter()).copied());

        let word = Word {
            words: self.words.clone(),
            bits,
        };

        word.incr();

        word
    }

    pub fn slice(&self, lo: usize, hi: usize) -> Word {
        assert!(lo <= hi);

        let bits = self.bits[lo..=hi].iter().copied().collect();

        let word = Word {
            words: self.words.clone(),
            bits,
        };

        word.incr();

        word
    }

    pub fn rotl(&self, k: usize) -> Word {
        let k = k % self.width();

        let mut word = self.clone();

        word.rotl_assign(k);

        word
    }

    pub fn rotr(&self, k: usize) -> Word {
        let k = k % self.width();

        self.rotl(self.width() - k)
    }

    pub fn rotl_assign(&mut self, k: usize) {
        // Note that rust's "right" for slices increasing in slice position. Since this is a word
        // and conventially we see "right" to mean decreasing in bit position, we rotate the bit
        // slice right instead of left here.
        self.bits.rotate_right(k);
    }

    pub fn rotr_assign(&mut self, k: usize) {
        self.rotl_assign(self.width() - k);
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

impl TryFrom<&Word> for u64 {
    type Error = ();

    fn try_from(value: &Word) -> Result<u64, Self::Error> {
        let mut n = 0;

        for (i, bit) in value.bits.iter().enumerate() {
            if value.words.is_true(*bit) {
                n |= (1u64 << i);
            } else if !value.words.is_false(*bit) {
                return Err(());
            }
        }

        Ok(n)
    }
}

impl BitAnd for Word {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        (&self).bitand(&rhs)
    }
}

impl BitAnd<&Word> for &Word {
    type Output = Word;

    fn bitand(self, rhs: &Word) -> Self::Output {
        todo!()
    }
}

impl BitAndAssign for Word {
    fn bitand_assign(&mut self, rhs: Word) {
        self.bitand_assign(&rhs);
    }
}

impl BitAndAssign<&Word> for Word {
    fn bitand_assign(&mut self, rhs: &Word) {
        todo!()
    }
}

impl BitOr for Word {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        (&self).bitor(&rhs)
    }
}

impl BitOr<&Word> for &Word {
    type Output = Word;

    fn bitor(self, rhs: &Word) -> Self::Output {
        todo!()
    }
}

impl BitOrAssign for Word {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bitor_assign(&rhs);
    }
}

impl BitOrAssign<&Word> for Word {
    fn bitor_assign(&mut self, rhs: &Word) {
        todo!()
    }
}

impl BitXor for Word {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        (&self).bitxor(&rhs)
    }
}

impl BitXor<&Word> for &Word {
    type Output = Word;

    fn bitxor(self, rhs: &Word) -> Self::Output {
        todo!()
    }
}

impl BitXorAssign for Word {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bitor_assign(&rhs);
    }
}

impl BitXorAssign<&Word> for Word {
    fn bitxor_assign(&mut self, rhs: &Word) {
        todo!()
    }
}

impl Not for Word {
    type Output = Word;

    fn not(self) -> Self::Output {
        (&self).not()
    }
}

impl Not for &Word {
    type Output = Word;

    fn not(self) -> Self::Output {
        todo!()
    }
}

impl Shl<u64> for Word {
    type Output = Word;

    fn shl(self, k: u64) -> Self::Output {
        (&self).shl(k)
    }
}

impl Shl<u64> for &Word {
    type Output = Word;

    fn shl(self, k: u64) -> Self::Output {
        todo!()
    }
}

impl ShlAssign<u64> for Word {
    fn shl_assign(&mut self, k: u64) {
        todo!()
    }
}

impl Shr<u64> for &Word {
    type Output = Word;

    fn shr(self, k: u64) -> Self::Output {
        todo!()
    }
}

impl ShrAssign<u64> for Word {
    fn shr_assign(&mut self, k: u64) {
        todo!()
    }
}

impl Add for Word {
    type Output = Word;

    fn add(self, rhs: Word) -> Self::Output {
        (&self).add(&rhs)
    }
}

impl Add<&Word> for &Word {
    type Output = Word;

    fn add(self, rhs: &Word) -> Self::Output {
        todo!()
    }
}

impl AddAssign for Word {
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}

impl AddAssign<&Word> for Word {
    fn add_assign(&mut self, rhs: &Self) {
        todo!()
    }
}

impl Neg for Word {
    type Output = Word;

    fn neg(self) -> Word {
        (&self).neg()
    }
}

impl Neg for &Word {
    type Output = Word;

    fn neg(self) -> Word {
        todo!()
    }
}

impl Sub for Word {
    type Output = Word;

    fn sub(self, rhs: Self) -> Self::Output {
        (&self).sub(&rhs)
    }
}

impl Sub<&Word> for &Word {
    type Output = Word;

    fn sub(self, rhs: &Word) -> Self::Output {
        todo!()
    }
}

impl SubAssign for Word {
    fn sub_assign(&mut self, rhs: Word) {
        self.sub_assign(&rhs);
    }
}

impl SubAssign<&Word> for Word {
    fn sub_assign(&mut self, rhs: &Word) {
        todo!()
    }
}

impl Mul for Word {
    type Output = Word;

    fn mul(self, rhs: Self) -> Self::Output {
        (&self).mul(&rhs)
    }
}

impl Mul<&Word> for &Word {
    type Output = Word;

    fn mul(self, rhs: &Word) -> Self::Output {
        todo!()
    }
}

impl MulAssign for Word {
    fn mul_assign(&mut self, rhs: Word) {
        self.mul_assign(&rhs);
    }
}

impl MulAssign<&Word> for Word {
    fn mul_assign(&mut self, rhs: &Word) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gc_01() {
        let mut words = Words::new();

        let bits = {
            let a = words.var(8);
            a.bits.clone()
        };

        for bit in &bits {
            assert_eq!(words.0.borrow().bits[*bit as usize].0, 0);
        }

        let before = words.0.borrow().bits.len();
        let b = words.var(8);
        let after = words.0.borrow().bits.len();

        assert_eq!(before, after);
    }

    #[test]
    fn convert_01() {
        let mut words = Words::new();

        for k in 0..=u8::MAX {
            let w = words.from_u64(8, k as u64);
            let j = u64::try_from(&w).unwrap() as u8;

            assert_eq!(k, j);
        }
    }

    #[test]
    fn convert_02() {
        let mut words = Words::new();

        let w = words.var(8);

        assert!(u64::try_from(&w).is_err());
    }

    #[test]
    fn rotl_01() {
        let mut words = Words::new();

        for n in 0..=u8::MAX {
            let w = words.from_u64(8, n as u64);

            for k in 0..=8 {
                let w = w.rotl(k);
                let m = u64::try_from(&w).unwrap() as u8;

                assert_eq!(n.rotate_left(k as u32), m);
            }
        }
    }

    #[test]
    fn rotr_01() {
        let mut words = Words::new();

        for n in 0..=u8::MAX {
            let w = words.from_u64(8, n as u64);

            for k in 0..=8 {
                let w = w.rotr(k);
                let m = u64::try_from(&w).unwrap() as u8;

                assert_eq!(n.rotate_right(k as u32), m);
            }
        }
    }

    #[test]
    fn rotl_02() {
        let mut words = Words::new();

        for n in 0..=u8::MAX {
            for k in 0..=8 {
                let mut w = words.from_u64(8, n as u64);
                w.rotr_assign(k);
                let m = u64::try_from(&w).unwrap() as u8;

                assert_eq!(n.rotate_right(k as u32), m);
            }
        }
    }

    #[test]
    fn rotr_02() {
        let mut words = Words::new();

        for n in 0..=u8::MAX {
            for k in 0..=8 {
                let mut w = words.from_u64(8, n as u64);
                w.rotl_assign(k);
                let m = u64::try_from(&w).unwrap() as u8;

                assert_eq!(n.rotate_left(k as u32), m);
            }
        }
    }
}
