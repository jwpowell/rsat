use std::io::BufRead;

use crate::dimacs::Dimacs;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Unknown,
    Sat,
    Unsat,
}

pub struct Solver {
    clauses: Vec<Vec<i32>>,
    status: Status,
    stack: Vec<Frame>,
    assignments: Vec<Assignment>,
    next_clause_index: usize,
}

#[derive(Clone, Copy, Debug)]
struct Assignment {
    value: Option<bool>,
    resolve_frame: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    satisfied: bool,
    clause_index: usize,
    guess_count: usize,
    resolve_frame: Option<usize>,
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
        let mut solver = Solver {
            clauses: vec![],
            status: Status::Unknown,
            stack: vec![],
            assignments: vec![],
            next_clause_index: 0,
        };

        let mut max = 0;
        let mut literals = vec![];

        while dimacs.next(&mut literals) {
            literals.sort_unstable();
            literals.dedup();

            max = std::cmp::max(
                literals.iter().map(|l| l.abs() as usize).max().unwrap(),
                max,
            );

            solver.clauses.push(literals.clone());
        }

        solver.clauses.sort_unstable();
        solver.clauses.dedup();
        solver.clauses.sort_unstable_by_key(|clause| clause.len());

        solver.assignments.extend(
            std::iter::repeat_with(|| Assignment {
                value: None,
                resolve_frame: None,
            })
            .take(max + 1),
        );

        solver
    }
}
