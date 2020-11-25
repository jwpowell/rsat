#![allow(unused)]

use std::cell::RefCell;
use std::rc::Rc;

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

pub struct Formula(Rc<RefCell<_Formula>>);

impl Default for Formula {
    fn default() -> Formula {
        Formula::new()
    }
}

impl Formula {
    pub fn new() -> Formula {
        Formula(Rc::new(RefCell::new(_Formula {
            bits: vec![],
            gc_bits: vec![],
        })))
    }

    fn bit(&mut self, bit: Bit) -> u32 {
        let mut inner = self.0.borrow_mut();

        if let Some(id) = inner.gc_bits.pop() {
            id
        } else {
            inner.bits.push((0, bit));
            (inner.bits.len() - 1) as u32
        }
    }

    fn incr(&mut self, id: u32) {
        self.0.borrow_mut().bits[id as usize].0 += 1;
    }

    fn decr(&mut self, id: u32) {
        let mut inner = self.0.borrow_mut();
        let mut stack = vec![id];

        while let Some(id) = stack.pop() {
            inner.bits[id as usize].0 -= 1;

            if inner.bits[id as usize].0 == 0 {
                inner.gc_bits.push(id);

                match inner.bits[id as usize].1 {
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

    fn refc(&mut self, id: u32) -> u32 {
        let mut inner = self.0.borrow();

        inner.bits[id as usize].0
    }

    fn shallow_clone(&self) -> Formula {
        Formula(Rc::clone(&self.0))
    }

    pub fn word(&mut self, width: usize) -> Word {
        let mut ids = Vec::with_capacity(width);

        for _ in 0..width {
            ids.push(self.bit(Bit::Var))
        }

        Word {
            formula: self.shallow_clone(),
            ids,
        }
    }
}

pub struct Word {
    formula: Formula,
    ids: Vec<u32>,
}
