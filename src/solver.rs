use std::io::BufRead;

use crate::dimacs::Dimacs;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Unknown,
    Sat,
    Unsat,
}

pub struct Solver {
    status: Status,
}

impl Solver {
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn finished(&self) -> bool {
        self.status() != Status::Unknown
    }

    pub fn step(&mut self) {
        todo!()
    }

    pub fn from_dimacs<R: BufRead>(mut dimacs: Dimacs<R>) -> Solver {
        Solver {
            status: Status::Unknown,
        }
    }
}
