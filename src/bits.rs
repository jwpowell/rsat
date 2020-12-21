use std::cell::RefCell;
use std::rc::Rc;

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

    pub fn gc_inf(&self) -> (usize, usize, usize, usize) {
        let bits_cap = self.0.borrow().bits.capacity();
        let bits_len = self.0.borrow().bits.len();
        let gc_cap = self.0.borrow().garbage.capacity();
        let gc_len = self.0.borrow().garbage.len();

        (bits_len, bits_cap, gc_len, gc_cap)
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
}
