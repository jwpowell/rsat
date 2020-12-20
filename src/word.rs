use std::ops::*;

pub struct Word;

impl Word {
    pub fn width(&self) -> usize {
        todo!()
    }

    pub fn simplify(&mut self) {
        todo!()
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for Word {}

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
