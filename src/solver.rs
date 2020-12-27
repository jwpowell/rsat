use std::collections::HashSet;
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
    handled: HashSet<usize>,
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
        self.status != Status::Unknown
    }

    fn next_clause_index(&self) -> Option<usize> {
        if self.next_clause_index == self.clauses.len() {
            return None;
        } else {
            return Some(self.next_clause_index);
        }

        if self.handled.len() == self.clauses.len() {
            return None;
        }

        /*
        println!(
            "[{} @ {}] {:?}",
            clause_index,
            self.stack.len() - 1,
            self.clauses[clause_index]
        );
        */

        //println!("  {:?}", ls);

        let mut candidates = vec![];
        let mut none = None;

        for clause_index in 0..self.clauses.len() {
            let mut satisfied = false;
            let mut resolve_frame = None;
            let mut guess_count = 0;

            if self.handled.contains(&clause_index) {
                continue;
            }

            for i in 0..self.clauses[clause_index].len() {
                let literal = self.clauses[clause_index][i];
                let assignment = self.assignments[literal.abs() as usize];

                if let Some(value) = assignment.value {
                    if value == (literal > 0) {
                        satisfied = true;
                        break;
                    } else {
                        resolve_frame = std::cmp::max(resolve_frame, assignment.resolve_frame);
                    }
                } else {
                    guess_count += 1;
                }
            }

            if satisfied {
                continue;
            }

            if resolve_frame.is_none() {
                none = Some(clause_index);
            }

            if let Some(0) = resolve_frame {
                return Some(clause_index);
            }

            candidates.push((clause_index, resolve_frame));
        }

        if let Some(next) = candidates.iter().min_by_key(|c| c.1) {
            Some(next.0)
        } else {
            none
        }
    }

    fn mark_satisfied(&mut self) {
        self.stack.last_mut().unwrap().satisfied = true;
    }

    fn assign(&mut self, literal: i32, resolve_frame: Option<usize>) {
        self.assignments[literal.abs() as usize] = Assignment {
            value: Some(literal > 0),
            resolve_frame,
        };
    }

    fn force(&mut self) {
        let frame = self.stack.last().unwrap();

        let clause_index = frame.clause_index;
        let literal_index = frame.guess_count - 1;
        let resolve_frame = frame.resolve_frame;
        let literal = self.clauses[clause_index][literal_index];

        self.assign(literal, resolve_frame);
        //println!("  force: {}, resovle to {:?}", literal, resolve_frame);
    }

    fn guess(&mut self) {
        let frame = self.stack.last().unwrap();

        let clause_index = frame.clause_index;
        let literal_index = frame.guess_count - 1;
        let resolve_frame = frame.resolve_frame;
        let literal = self.clauses[clause_index][literal_index];

        self.assign(literal, Some(self.stack.len() - 1));
        //println!("  guess: {}", literal);
    }

    fn invert(&mut self) {
        let frame = self.stack.last().unwrap();

        let clause_index = frame.clause_index;
        let literal_index = frame.guess_count - 1;
        let resolve_frame = frame.resolve_frame;
        let literal = self.clauses[clause_index][literal_index];

        self.assign(-literal, resolve_frame);
    }

    fn unassign(&mut self) {
        let frame = self.stack.last().unwrap();

        if frame.satisfied || frame.guess_count == 0 {
            return;
        }

        let clause_index = frame.clause_index;
        let literal_index = frame.guess_count - 1;
        let literal = self.clauses[clause_index][literal_index];

        self.handled.remove(&clause_index);

        self.assignments[literal.abs() as usize].value = None;
    }

    fn back_jump(&mut self, frame_index: usize) {
        while self.stack.len() > frame_index + 1 {
            self.unassign();
            self.stack.pop();
        }

        self.invert();

        let mut frame = self.stack.last().unwrap().clone();

        frame.guess_count -= 1;
        self.next_clause_index = frame.clause_index;

        self.stack.push(frame);
    }

    pub fn step(&mut self) {
        if self.finished() {
            return;
        }

        if self.next_clause_index().is_none() {
            self.status = Status::Sat;
            return;
        }

        self.stack.push(Frame {
            satisfied: false,
            clause_index: self.next_clause_index().unwrap(),
            guess_count: 0,
            resolve_frame: None,
        });

        let clause_index = self.stack.last().unwrap().clause_index;

        self.handled.insert(clause_index);

        let mut satisfied = false;
        let mut resolve_frame = None;
        let mut guess_count = 0;

        /*
        println!(
            "[{} @ {}] {:?}",
            clause_index,
            self.stack.len() - 1,
            self.clauses[clause_index]
        );
        */

        //println!("  {:?}", ls);

        for i in 0..self.clauses[clause_index].len() {
            let literal = self.clauses[clause_index][i];
            let assignment = self.assignments[literal.abs() as usize];

            if let Some(value) = assignment.value {
                if value == (literal > 0) {
                    satisfied = true;
                    break;
                } else {
                    resolve_frame = std::cmp::max(resolve_frame, assignment.resolve_frame);
                }
            } else {
                self.clauses[clause_index].swap(i, guess_count);
                guess_count += 1;
            }
        }

        {
            let frame = self.stack.last_mut().unwrap();
            frame.guess_count = guess_count;
            frame.satisfied = satisfied;
            frame.resolve_frame = resolve_frame;

            //println!("{:#?}", frame);
        }

        if satisfied {
            self.next_clause_index += 1;
            return;
        }

        if guess_count == 0 {
            if let Some(index) = resolve_frame {
                self.back_jump(index);
            } else {
                self.status = Status::Unsat;
                return;
            }
            //println!("  {:?}", ls);
        }

        let frame = self.stack.last_mut().unwrap();

        if frame.guess_count == 1 {
            self.force();
        } else {
            self.guess();
        }

        self.next_clause_index += 1;

        //let mut ls = vec![];
        //self.assignments(&mut ls);
        //assert!(self.check(&ls));
    }

    pub fn from_dimacs<R: BufRead>(mut dimacs: Dimacs<R>) -> Solver {
        let mut solver = Solver {
            clauses: vec![],
            status: Status::Unknown,
            stack: vec![],
            assignments: vec![],
            next_clause_index: 0,
            handled: HashSet::new(),
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

    pub fn assignments(&mut self, literals: &mut Vec<i32>) {
        for (v, assignment) in self.assignments.iter().enumerate() {
            let literal = v as i32;

            if let Some(value) = assignment.value {
                if value {
                    literals.push(literal);
                } else {
                    literals.push(-literal);
                }
            }
        }
    }

    pub fn check(&self, literals: &[i32]) -> bool {
        let set: HashSet<i32> = literals.iter().copied().collect();

        for (i, clause) in self.clauses.iter().enumerate() {
            let mut satisfied = false;

            for literal in clause {
                if set.contains(&literal) {
                    satisfied = true;
                    break;
                }
            }

            if !satisfied {
                println!(
                    "[{}] {:?}",
                    i,
                    clause
                        .iter()
                        .map(|literal| (*literal, set.contains(literal), set.contains(&-literal)))
                        .collect::<Vec<(i32, bool, bool)>>()
                );

                return false;
            }
        }

        true
    }
}
