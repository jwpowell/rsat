use std::cell::{RefCell, RefMut};
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Bit {
    Var,
    Val(bool),

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

    pub fn zero(&mut self, width: usize) -> Word {
        self.from_u64(0, width)
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
            let id = xor_help(&mut inner, a.ids[i], b.ids[i]);

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

    pub fn addc(&mut self, a: &Word, b: &Word) -> (Word, Word) {
        assert_eq!(a.width(), b.width());

        let mut inner = self.0.borrow_mut();
        let mut ids = Vec::with_capacity(a.width());
        let mut carry = inner.val(false);

        for i in 0..a.width() {
            let (s, c) = full_adder_help(&mut inner, a.ids[i], b.ids[i], carry);
            inner.decr(carry);
            carry = c;

            ids.push(s);
        }

        (
            Word {
                formula: self.shallow_clone(),
                ids,
            },
            Word {
                formula: self.shallow_clone(),
                ids: vec![carry],
            },
        )
    }

    pub fn add(&mut self, a: &Word, b: &Word) -> Word {
        self.addc(a, b).0
    }

    pub fn slice(&self, a: &Word, lo: usize, hi: usize) -> Word {
        let mut inner = self.0.borrow_mut();
        let ids: Vec<u32> = a.ids[lo..=hi].iter().copied().collect();

        for id in &ids {
            inner.incr(*id);
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn concat(&self, a: &Word, b: &Word) -> Word {
        let mut inner = self.0.borrow_mut();
        let ids: Vec<u32> = a.ids.iter().chain(b.ids.iter()).copied().collect();

        for id in &ids {
            inner.incr(*id);
        }

        assert_eq!(ids.len(), a.width() + b.width());

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn cond(&mut self, test: &Word, yes: &Word, no: &Word) -> Word {
        assert_eq!(yes.width(), no.width());
        assert_eq!(test.width(), 1);

        let mut inner = self.0.borrow_mut();
        let mut ids = Vec::with_capacity(yes.width());
        let test_bit = test.ids[0];
        let t1 = inner.not(test_bit);

        for i in 0..yes.width() {
            let t2 = inner.and(test_bit, yes.ids[i]);
            let t3 = inner.and(t1, no.ids[i]);
            let id = inner.or(t2, t3);

            inner.decr(t2);
            inner.decr(t3);

            ids.push(id);
        }

        inner.decr(t1);

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }

    pub fn mul(&mut self, a: &Word, b: &Word) -> Word {
        assert_eq!(a.width(), b.width());

        let zero = self.zero(a.width());
        let mut sum = zero.clone();

        for i in 0..a.width() {
            let test = self.slice(b, i, i);
            let v = self.cond(&test, &a, &zero);
            let v = self.shl(&v, i);
            sum = self.add(&sum, &v);
        }

        sum
    }

    pub fn rotl(&mut self, a: &Word, k: usize) -> Word {
        let x = self.shl(a, k);
        let y = self.shr(a, a.width() - k);

        self.or(&x, &y)
    }

    pub fn rotr(&mut self, a: &Word, k: usize) -> Word {
        self.rotl(a, a.width() - k)
    }

    pub fn fold_and(&mut self, a: &Word) -> Word {
        let mut inner = self.0.borrow_mut();

        let mut value = inner.val(true);

        for id in &a.ids {
            let t1 = value;
            value = inner.and(value, *id);
            inner.decr(t1);
        }

        Word {
            formula: self.shallow_clone(),
            ids: vec![value],
        }
    }

    pub fn fold_or(&mut self, a: &Word) -> Word {
        let mut inner = self.0.borrow_mut();

        let mut value = inner.val(false);

        for id in &a.ids {
            let t1 = value;
            value = inner.or(value, *id);
            inner.decr(t1);
        }

        Word {
            formula: self.shallow_clone(),
            ids: vec![value],
        }
    }

    pub fn fold_xor(&mut self, a: &Word) -> Word {
        let mut inner = self.0.borrow_mut();

        let mut value = inner.val(false);

        for id in &a.ids {
            let t1 = value;
            value = xor_help(&mut inner, value, *id);
            inner.decr(t1);
        }

        Word {
            formula: self.shallow_clone(),
            ids: vec![value],
        }
    }

    // lhs[n] && !rhs[n] or
    pub fn less_than(&mut self, a: &Word, b: &Word) -> Word {
        assert_eq!(a.width(), b.width());

        let mut inner = self.0.borrow_mut();
        let mut value = inner.val(false);

        for i in (0..a.width()).into_iter() {
            // a' AND b
            let t2 = inner.not(a.ids[i]);
            let t3 = inner.and(t2, b.ids[i]);

            // NOT (a AND b')
            let t4 = inner.not(b.ids[i]);
            let t5 = inner.and(a.ids[i], t4);
            let t6 = inner.not(t5);

            // (a' AND b) OR NOT(a AND b') AND value
            let t7 = inner.and(t6, value);
            let t8 = value;
            value = inner.or(t3, t7);

            inner.decr(t2);
            inner.decr(t3);
            inner.decr(t4);
            inner.decr(t5);
            inner.decr(t6);
            inner.decr(t7);
            inner.decr(t8);
        }

        Word {
            formula: self.shallow_clone(),
            ids: vec![value],
        }
    }

    pub fn equal_to(&mut self, a: &Word, b: &Word) -> Word {
        let c = self.xor(a, b);
        let d = self.fold_or(&c);

        self.not(&d)
    }

    pub fn greater_than(&mut self, a: &Word, b: &Word) -> Word {
        let t1 = self.less_than(a, b);
        let t2 = self.equal_to(a, b);
        let t3 = self.or(&t1, &t2);

        self.not(&t3)
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

fn xor_help(inner: &mut RefMut<_Formula>, a: u32, b: u32) -> u32 {
    let t1 = inner.not(a);
    let t2 = inner.not(b);
    let t3 = inner.and(t1, b);
    let t4 = inner.and(a, t2);

    let id = inner.or(t3, t4);

    inner.decr(t1);
    inner.decr(t2);
    inner.decr(t3);
    inner.decr(t4);

    id
}

fn full_adder_help(inner: &mut RefMut<_Formula>, a: u32, b: u32, c: u32) -> (u32, u32) {
    let t1 = xor_help(inner, a, b);
    let s = xor_help(inner, t1, c);

    let t2 = inner.and(a, b);
    let t3 = inner.and(c, t1);
    let c = inner.or(t2, t3);

    inner.decr(t1);
    inner.decr(t2);
    inner.decr(t3);

    (s, c)
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
    fn add_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.add(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = (a + b) & 0x0f;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn mul_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.mul(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = (a * b) & 0x0f;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn less_than_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.less_than(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                println!("{} {}", a, b);
                let c_expected = if a < b { 1 } else { 0 };

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn equal_to_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.equal_to(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                println!("{} {}", a, b);
                let c_expected = if a == b { 1 } else { 0 };

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn greater_than_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            let x = formula.from_u64(a, 4);

            for b in 0..=15 {
                let y = formula.from_u64(b, 4);
                let z = formula.greater_than(&x, &y);

                let c_actual = formula.try_to_u64(&z).unwrap();
                println!("{} {}", a, b);
                let c_expected = if a > b { 1 } else { 0 };

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn mul_02() {
        let mut formula = Formula::new();

        let a = formula.from_u64(12_345, 64);
        let b = formula.from_u64(54_321, 64);

        let c = formula.mul(&a, &b);

        assert_eq!(formula.try_to_u64(&c), Some(670_592_745));
    }

    #[test]
    fn mul_03() {
        let mut formula = Formula::new();

        let a = formula.word(64);
        let b = formula.word(64);

        let c = formula.mul(&a, &b);

        println!("{}", formula.gc_live_count());
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

    #[test]
    fn rotl_01() {
        let mut formula = Formula::new();

        for a in 0..=255 {
            for k in 0..=8 {
                let x = formula.from_u64(a, 8);
                let z = formula.rotl(&x, k);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = (a as u8).rotate_left(k as u32) as u64;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn rotr_01() {
        let mut formula = Formula::new();

        for a in 0..=255 {
            for k in 0..=8 {
                let x = formula.from_u64(a, 8);
                let z = formula.rotr(&x, k);

                let c_actual = formula.try_to_u64(&z).unwrap();
                let c_expected = (a as u8).rotate_right(k as u32) as u64;

                assert_eq!(c_actual, c_expected);
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }

    #[test]
    fn cond_01() {
        let mut formula = Formula::new();

        for a in 0..=15 {
            for b in 0..=15 {
                for t in &[0, 1] {
                    let yes = formula.from_u64(a, 4);
                    let no = formula.from_u64(b, 4);
                    let test = formula.from_u64(*t, 1);
                    let cond = formula.cond(&test, &yes, &no);

                    let res_actual = formula.try_to_u64(&cond).unwrap();
                    let res_expected = if *t == 1 { a } else { b };

                    assert_eq!(res_actual, res_expected);
                }
            }
        }

        assert_eq!(formula.gc_live_count(), 0);
    }
}
