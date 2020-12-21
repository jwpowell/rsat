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
        self.alloc_bit(Bit::Var)
    }

    /// Create a boolean value
    pub fn val(&self, v: bool) -> u32 {
        self.alloc_bit(Bit::Val(v))
    }

    /// Create a conjunction of two expressions
    pub fn and(&self, a: u32, b: u32) -> u32 {
        self.alloc_bit(Bit::And(a, b))
    }

    /// Create a disjunction of two expressions
    pub fn or(&self, a: u32, b: u32) -> u32 {
        self.alloc_bit(Bit::Or(a, b))
    }

    /// Create the complement of an expression
    pub fn not(&self, a: u32) -> u32 {
        self.alloc_bit(Bit::Not(a))
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

    pub fn len(&self) -> usize {
        self.0.borrow().bits.len()
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

        assert_eq!(bits.refcount(a), 0);
        bits.incr(a);
        assert_eq!(bits.refcount(a), 1);

        let b = bits.var();
        bits.incr(b);

        let c = bits.and(a, b);
        bits.incr(c);

        assert_eq!(bits.refcount(a), 1);
        assert_eq!(bits.refcount(b), 1);
        assert_eq!(bits.refcount(c), 1);

        bits.decr(c);

        assert_eq!(bits.refcount(a), 0);
        assert_eq!(bits.refcount(b), 0);
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
