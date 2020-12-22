use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Mul,
    MulAssign, Neg, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use std::cell::RefCell;
use std::rc::Rc;

use std::convert::{TryFrom, TryInto};

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

impl MulAssign<&Word> for Word {
    fn mul_assign(&mut self, rhs: &Word) {
        let c = &*self * rhs;
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

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Bit {
    Unused,
    Var,
    Val(bool),

    And(u32, u32),
    Or(u32, u32),
    Not(u32),
}

struct _Bits {
    bits: Vec<(u32, Bit)>,
    garbage: Vec<u32>,
}

pub struct Bits(Rc<RefCell<_Bits>>);

impl Default for Bits {
    fn default() -> Bits {
        Bits::new()
    }
}

impl Bits {
    /// Creates a new container of boolean expressions.
    pub fn new() -> Bits {
        Bits(Rc::new(RefCell::new(_Bits {
            bits: vec![],
            garbage: vec![],
        })))
    }

    /// Create a boolean variable
    pub fn var(&self) -> u32 {
        let c = self.alloc_bit(Bit::Var);

        self.incr(c);

        c
    }

    /// Create a boolean value
    pub fn val(&self, v: bool) -> u32 {
        let c = self.alloc_bit(Bit::Val(v));

        self.incr(c);

        c
    }

    /// Create a conjunction of two expressions
    pub fn and(&self, a: u32, b: u32) -> u32 {
        if self.is_false(a) || self.is_true(b) {
            self.incr(a);
            return a;
        }

        if self.is_true(a) || self.is_false(b) {
            self.incr(b);
            return b;
        }

        let c = self.alloc_bit(Bit::And(a, b));

        self.incr(a);
        self.incr(b);
        self.incr(c);

        c
    }

    /// Create a disjunction of two expressions
    pub fn or(&self, a: u32, b: u32) -> u32 {
        if self.is_false(a) || self.is_true(b) {
            self.incr(b);
            return b;
        }

        if self.is_true(a) || self.is_false(b) {
            self.incr(a);
            return a;
        }

        let c = self.alloc_bit(Bit::Or(a, b));

        self.incr(a);
        self.incr(b);
        self.incr(c);

        c
    }

    /// Create the complement of an expression
    pub fn not(&self, a: u32) -> u32 {
        if self.is_false(a) {
            return self.val(true);
        }

        if self.is_true(a) {
            return self.val(false);
        }

        let c = self.alloc_bit(Bit::Not(a));

        self.incr(a);
        self.incr(c);

        c
    }

    pub fn xor(&self, a: u32, b: u32) -> u32 {
        let t1 = self.not(a);
        let t2 = self.not(b);

        let t3 = self.and(t1, b);
        let t4 = self.and(a, t2);

        let c = self.or(t3, t4);

        self.decr(t1);
        self.decr(t2);
        self.decr(t3);
        self.decr(t4);

        c
    }

    pub fn full_adder(&self, a: u32, b: u32, c_in: u32) -> (u32, u32) {
        let t1 = self.xor(a, b);
        let s = self.xor(t1, c_in);
        let t2 = self.and(c_in, t1);
        let t3 = self.and(a, b);
        let c = self.or(t2, t3);

        self.decr(t3);
        self.decr(t2);
        self.decr(t1);

        (s, c)
    }

    pub fn cond(&self, test: u32, yes: u32, no: u32) -> u32 {
        let t1 = self.and(test, yes);
        let t2 = self.not(test);
        let t3 = self.and(t2, no);
        let c = self.or(t1, t3);

        self.decr(t1);
        self.decr(t2);
        self.decr(t3);

        c
    }

    /// Gets the `Bit` expression for the given expression
    pub fn get(&self, id: u32) -> Bit {
        self.0.borrow().bits[id as usize].1
    }

    /// Sets the expression.
    pub fn set(&self, id: u32, bit: Bit) {
        todo!()
    }

    /// Returns the reference counter for an expression
    pub fn refcount(&self, id: u32) -> u32 {
        self.0.borrow().bits[id as usize].0
    }

    /// Increment the reference counter for an expression
    pub fn incr(&self, id: u32) {
        let mut inner = self.0.borrow_mut();
        inner.bits[id as usize].0 += 1;
    }

    ///Decrement the reference counter for an expression. If the reference counter reaches zero,
    ///garbage collect the ID and recursively decrement an other referenced expressions.
    pub fn decr(&self, id: u32) {
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

    fn alloc_bit(&self, bit: Bit) -> u32 {
        let mut inner = self.0.borrow_mut();

        if let Some(id) = inner.garbage.pop() {
            inner.bits[id as usize] = (0, bit);
            return id;
        }

        let id = inner.bits.len() as u32;
        inner.bits.push((0, bit));

        id
    }

    pub fn is_val(&self, id: u32, v: bool) -> bool {
        matches!(self.get(id), Bit::Val(w) if w == v)
    }

    pub fn is_false(&self, id: u32) -> bool {
        self.is_val(id, false)
    }

    pub fn is_true(&self, id: u32) -> bool {
        self.is_val(id, true)
    }

    pub fn ptr_eq(&self, other: &Bits) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn size(&self) -> usize {
        self.0.borrow().bits.len()
    }

    pub fn depth(&self, id: u32) -> u32 {
        let bit = self.get(id);

        match bit {
            Bit::Unused => 0,
            Bit::Var => 1,
            Bit::Val(_) => 1,
            Bit::And(l, r) => 1 + std::cmp::max(self.depth(l), self.depth(r)),
            Bit::Or(l, r) => 1 + std::cmp::max(self.depth(l), self.depth(r)),
            Bit::Not(e) => 1 + self.depth(e),
        }
    }
}

impl Clone for Bits {
    fn clone(&self) -> Bits {
        Bits(Rc::clone(&self.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const MAX: u64 = 15;
    const BITS: usize = 4;

    #[test]
    fn gc_01() {
        let mut bits = Bits::new();

        let a = bits.var();
        assert_eq!(bits.refcount(a), 1);

        let b = bits.var();

        let c = bits.and(a, b);

        assert_eq!(bits.refcount(a), 2);
        assert_eq!(bits.refcount(b), 2);
        assert_eq!(bits.refcount(c), 1);

        bits.decr(c);

        assert_eq!(bits.refcount(a), 1);
        assert_eq!(bits.refcount(b), 1);
        assert_eq!(bits.refcount(c), 0);
    }

    #[test]
    fn gc_02() {
        let mut bits = Bits::new();
        let mut xs = vec![];

        let a = bits.var();

        xs.push(a);

        for i in 1..xs.len() {
            xs.push(xs[i - 1])
        }
    }

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
    fn mul_02() {
        let bits = Bits::new();

        for k in 0..=MAX {
            for j in 0..=MAX {
                let a = Word::from_u64(&bits, BITS, k);
                let b = Word::from_u64(&bits, BITS, j);
                let mut c = a.clone();
                c *= &b;

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
