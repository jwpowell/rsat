use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, MulAssign,
    Neg, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use crate::bits::{Bit, Bits};

use std::convert::TryFrom;
use std::fmt;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Word {
    words: Bits,
    bits: Vec<u32>,
}

impl Clone for Word {
    fn clone(&self) -> Word {
        todo!()
    }
}

impl Drop for Word {
    fn drop(&mut self) {
        todo!()
    }
}

impl Word {
    pub fn width(&self) -> usize {
        self.bits.len()
    }

    pub fn simplify(&mut self) {
        todo!()
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
