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
    pub fn new() -> Bits {
        Bits(Rc::new(RefCell::new(_Bits {
            bits: vec![],
            garbage: vec![],
        })))
    }

    pub fn var(&self) -> u32 {
        self.alloc_bit(Bit::Var)
    }

    pub fn val(&self, v: bool) -> u32 {
        self.alloc_bit(Bit::Val(v))
    }

    pub fn and(&self, a: u32, b: u32) -> u32 {
        self.alloc_bit(Bit::And(a, b))
    }

    pub fn or(&self, a: u32, b: u32) -> u32 {
        self.alloc_bit(Bit::Or(a, b))
    }

    pub fn not(&self, a: u32) -> u32 {
        self.alloc_bit(Bit::Not(a))
    }

    pub fn incr(&self, id: u32) {
        let mut inner = self.0.borrow_mut();
        inner.bits[id as usize].0 += 1;
    }

    pub fn get(&self, id: u32) -> Bit {
        self.0.borrow().bits[id as usize].1
    }

    pub fn refcount(&self, id: u32) -> u32 {
        self.0.borrow().bits[id as usize].0
    }

    pub fn set(&self, id: u32, bit: Bit) {
        todo!()
    }

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
            inner.bits[id as usize] = (1, bit);
            return id;
        }

        let id = inner.bits.len() as u32;
        inner.bits.push((1, bit));

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
}

impl Clone for Bits {
    fn clone(&self) -> Bits {
        Bits(Rc::clone(&self.0))
    }
}
