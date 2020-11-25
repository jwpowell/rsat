#![allow(unused)]

use std::{cell::RefCell, ops::DivAssign};
use std::{ops::Rem, rc::Rc};

#[derive(Clone, Copy)]
enum Bit {
    Val(bool),
    Var,

    And(u32, u32),
    Or(u32, u32),
    Not(u32),
}

struct _Formula {
    bits: Vec<(u32, Bit)>,
    gc_bits: Vec<u32>,
}

impl _Formula {
    /// Create an empty formula
    fn new() -> _Formula {
        _Formula {
            bits: vec![],
            gc_bits: vec![],
        }
    }

    /// Allocates a bit in the `_Formula` and returns its ID.
    fn insert(&mut self, bit: Bit) -> u32 {
        if let Some(id) = self.gc_bits.pop() {
            self.bits[id as usize] = (0, bit);
            self.incr(id);

            id
        } else {
            let id = self.bits.len() as u32;
            self.bits.push((0, bit));
            self.incr(id);

            id
        }
    }

    fn var(&mut self) -> u32 {
        self.insert(Bit::Var)
    }

    fn val(&mut self, v: bool) -> u32 {
        self.insert(Bit::Val(v))
    }

    fn and(&mut self, a: u32, b: u32) -> u32 {
        if self.is_false(a) || self.is_true(b) {
            self.incr(a);
            return a;
        }

        if self.is_true(a) || self.is_false(b) {
            self.incr(b);
            return b;
        }

        self.incr(a);
        self.incr(b);
        self.insert(Bit::And(a, b))
    }

    fn or(&mut self, a: u32, b: u32) -> u32 {
        if self.is_false(a) || self.is_true(b) {
            self.incr(b);
            return b;
        }

        if self.is_true(a) || self.is_false(b) {
            self.incr(a);
            return a;
        }

        self.incr(a);
        self.incr(b);
        self.insert(Bit::Or(a, b))
    }

    fn not(&mut self, a: u32) -> u32 {
        if self.is_false(a) {
            return self.val(true);
        }

        if self.is_true(a) {
            return self.val(false);
        }

        self.incr(a);
        self.insert(Bit::Not(a))
    }

    fn is_val(&self, id: u32, v: bool) -> bool {
        matches!(self.bits[id as usize].1, Bit::Val(u) if u == v)
    }

    fn is_true(&self, id: u32) -> bool {
        self.is_val(id, true)
    }

    fn is_false(&self, id: u32) -> bool {
        self.is_val(id, false)
    }

    /// Increments the refcount for the bit with the provided ID.
    fn incr(&mut self, id: u32) {
        self.bits[id as usize].0 += 1;
    }

    /// Decrements the refcount for the bit with the provided ID. This method eagerly garbage
    /// collects the provided bit and all bits transitively referenced.
    fn decr(&mut self, id: u32) {
        let mut stack = vec![id];

        while let Some(id) = stack.pop() {
            self.bits[id as usize].0 -= 1;

            if self.bits[id as usize].0 == 0 {
                self.gc_bits.push(id);

                match self.bits[id as usize].1 {
                    Bit::And(a, b) => {
                        stack.push(a);
                        stack.push(b);
                    }

                    Bit::Or(a, b) => {
                        stack.push(a);
                        stack.push(b);
                    }

                    Bit::Not(a) => {
                        stack.push(a);
                    }

                    _ => {}
                }
            }
        }
    }

    fn refc(&self, id: u32) -> u32 {
        self.bits[id as usize].0
    }
}

pub struct Formula(Rc<RefCell<_Formula>>);

impl Default for Formula {
    fn default() -> Formula {
        Formula::new()
    }
}

impl Formula {
    pub fn new() -> Formula {
        Formula(Rc::new(RefCell::new(_Formula::new())))
    }

    fn shallow_clone(&self) -> Formula {
        Formula(Rc::clone(&self.0))
    }

    pub fn word(&mut self, width: usize) -> Word {
        let mut ids = Vec::with_capacity(width);
        let mut inner = self.0.borrow_mut();

        for _ in 0..width {
            let id = inner.var();
            ids.push(id);
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn from_u64(&mut self, mut n: u64, width: usize) -> Word {
        let mut ids = Vec::with_capacity(width);
        let mut inner = self.0.borrow_mut();

        for _ in 0..width {
            let id = if n % 2 == 0 {
                inner.val(false)
            } else {
                inner.val(true)
            };

            ids.push(id);

            n /= 2;
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn try_to_u64(&self, a: &Word) -> Option<u64> {
        let inner = self.0.borrow();

        if a.width() > 64 {
            return None;
        }

        let mut value = 0;

        for (i, id) in a.ids.iter().enumerate() {
            if inner.is_true(*id) {
                value |= 1u64 << i;
            } else if !inner.is_false(*id) {
                return None;
            }
        }

        Some(value)
    }

    pub fn and(&mut self, a: &Word, b: &Word) -> Word {
        assert_eq!(a.width(), b.width());

        let mut inner = self.0.borrow_mut();
        let mut ids = Vec::with_capacity(a.width());

        for i in 0..a.width() {
            let id = inner.and(a.ids[i], b.ids[i]);
            ids.push(id);
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn or(&mut self, a: &Word, b: &Word) -> Word {
        assert_eq!(a.width(), b.width());

        let mut inner = self.0.borrow_mut();
        let mut ids = Vec::with_capacity(a.width());

        for i in 0..a.width() {
            let id = inner.or(a.ids[i], b.ids[i]);
            ids.push(id);
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn xor(&mut self, a: &Word, b: &Word) -> Word {
        assert_eq!(a.width(), b.width());

        let mut inner = self.0.borrow_mut();
        let mut ids = Vec::with_capacity(a.width());

        for i in 0..a.width() {
            let t1 = inner.not(a.ids[i]);
            let t2 = inner.not(b.ids[i]);
            let t3 = inner.and(t1, b.ids[i]);
            let t4 = inner.and(a.ids[i], t2);

            let id = inner.or(t3, t4);

            inner.decr(t1);
            inner.decr(t2);
            inner.decr(t3);
            inner.decr(t4);

            ids.push(id);
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn not(&mut self, a: &Word) -> Word {
        let mut inner = self.0.borrow_mut();
        let mut ids = Vec::with_capacity(a.width());

        for i in 0..a.width() {
            let id = inner.not(a.ids[i]);
            ids.push(id);
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn shl(&mut self, a: &Word, k: usize) -> Word {
        let mut inner = self.0.borrow_mut();

        let mut ids = Vec::with_capacity(a.width());

        for id in std::iter::repeat_with(|| inner.val(false)).take(k) {
            ids.push(id);
        }

        for id in &a.ids[..a.width() - k] {
            inner.incr(*id);
            ids.push(*id);
        }

        assert_eq!(a.width(), ids.len());

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn shr(&mut self, a: &Word, k: usize) -> Word {
        let mut inner = self.0.borrow_mut();

        let mut ids = Vec::with_capacity(a.width());

        for id in &a.ids[k..] {
            inner.incr(*id);
            ids.push(*id);
        }

        for id in std::iter::repeat_with(|| inner.val(false)).take(k) {
            ids.push(id);
        }

        assert_eq!(a.width(), ids.len());

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    fn gc_live_count(&self) -> usize {
        let mut count = 0;

        for (refc, _) in self.0.borrow().bits.iter() {
            if *refc > 0 {
                count += 1;
            }
        }

        count
    }
}

pub struct Word {
    formula: Formula,
    ids: Vec<u32>,
}

impl Word {
    pub fn width(&self) -> usize {
        self.ids.len()
    }
}

impl Clone for Word {
    fn clone(&self) -> Word {
        let mut inner = self.formula.0.borrow_mut();
        let mut ids = self.ids.clone();

        for id in &ids {
            inner.incr(*id);
        }

        Word {
            formula: self.formula.shallow_clone(),
            ids,
        }
    }
}

impl Drop for Word {
    fn drop(&mut self) {
        let mut inner = self.formula.0.borrow_mut();

        for id in self.ids.iter() {
            inner.decr(*id);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn word_convert_u64_01() {
        let mut formula = Formula::new();

        for n in 0..=255 {
            let a = formula.from_u64(n, 8);
            assert_eq!(formula.try_to_u64(&a), Some(n));
        }
    }

    #[test]
    fn gc_01() {
        let mut formula = Formula::new();

        let a = formula.word(32);

        {
            let b = formula.word(32);
            assert_eq!(formula.gc_live_count(), 64);
        }

        assert_eq!(formula.gc_live_count(), 32);
    }

    #[test]
    fn and_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.and(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = a & b;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn or_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.or(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = a | b;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn xor_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.xor(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = a ^ b;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn not_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);
            let z = formula.not(&x);

            let c_actual = formula.try_to_u64(&z).unwrap();
            let c_expected = (!a) & 0x0f;

            assert_eq!(c_actual, c_expected);
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn shl_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            for k in 0..=3 {
                let x = formula.from_u64(a, 4);
                let z = formula.shl(&x, k);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = (a << k) & 0xf;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn shr_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            for k in 0..=3 {
                let x = formula.from_u64(a, 4);
                let z = formula.shr(&x, k);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = (a >> k) & 0xf;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }
}
