use std::io::BufRead;

use crate::dimacs::Dimacs;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SolverStatus {
    Unknown,
    Sat,
    Unsat,
}

#[derive(Clone, Copy)]
struct Frame {
    guess_index: usize,

    resolve_literal: i32,
    resolve_depth: usize,
}

impl Frame {
    fn new() -> Frame {
        Frame {
            guess_index: 0,
            resolve_depth: 0,
            resolve_literal: 0,
        }
    }
}

struct Assignment {
    value: bool,
    depth: usize,

    resolve_literal: i32,
    resolve_depth: usize,
}

impl Assignment {
    fn value(&self, literal: i32) -> bool {
        self.value == (literal > 0)
    }
}

pub struct Solver {
    clauses: Vec<Vec<i32>>,

    stack: Vec<Frame>,
    assignments: Vec<Assignment>,
    status: SolverStatus,
}

impl Solver {
    pub fn new<R>(mut dimacs: Dimacs<R>) -> Solver
    where
        R: BufRead,
    {
        let mut solver = Solver {
            clauses: vec![],
            stack: vec![],
            assignments: vec![],
            status: SolverStatus::Unknown,
        };

        let mut literals = vec![];
        let mut var_max: usize = 0;

        while dimacs.next(&mut literals) {
            var_max = std::cmp::max(
                literals
                    .iter()
                    .copied()
                    .map(|l| l.abs() as usize)
                    .max()
                    .unwrap() as usize,
                var_max,
            );

            solver.clauses.push(literals.clone());
            solver.clauses.last_mut().unwrap().sort_unstable();
            solver.clauses.last_mut().unwrap().dedup();

            literals.clear();
        }

        solver.clauses.sort_unstable();
        solver.clauses.dedup();
        solver.clauses.sort_unstable_by_key(|clause| clause.len());

        solver.assignments.clear();
        solver.assignments.extend(
            std::iter::repeat_with(|| Assignment {
                value: false,
                depth: usize::MAX,
                resolve_depth: 0,
                resolve_literal: 0,
            })
            .take(var_max + 1),
        );

        solver
    }

    fn top(&self) -> &Frame {
        self.stack.last().unwrap()
    }

    fn top_mut(&mut self) -> &mut Frame {
        self.stack.last_mut().unwrap()
    }

    fn depth(&self) -> usize {
        self.stack.len()
    }

    fn assign(&mut self, literal: i32, resolve_literal: i32, resolve_depth: usize) {
        debug_assert_ne!(literal, 0);

        let current_depth = self.depth();
        let assignment = self.assignment_mut(literal);

        assignment.value = literal > 0;
        assignment.depth = current_depth;
        assignment.resolve_literal = resolve_literal;
        assignment.resolve_depth = resolve_depth;
    }

    fn force(&mut self, literal: i32) {
        self.assign(
            literal,
            self.top().resolve_literal,
            self.top().resolve_depth,
        );
    }

    fn guess(&mut self, literal: i32) {
        self.assign(literal, -literal, self.depth())
    }

    fn assignment(&self, literal: i32) -> &Assignment {
        let variable = literal.abs() as usize;
        &self.assignments[variable]
    }

    fn assignment_mut(&mut self, literal: i32) -> &mut Assignment {
        let variable = literal.abs() as usize;
        &mut self.assignments[variable]
    }

    fn is_assigned(&self, literal: i32) -> bool {
        self.assignment(literal).depth <= self.depth()
            && self.assignment(literal).value == (literal > 0)
    }

    fn resolution(&self, literal: i32) -> (usize, i32) {
        let assignment = self.assignment(literal);

        (assignment.resolve_depth, assignment.resolve_literal)
    }

    fn clause(&self) -> &[i32] {
        &self.clauses[self.depth() - 1]
    }

    fn push(&mut self) {
        self.stack.push(Frame::new());
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn pop_to(&mut self, depth: usize) {
        self.stack.truncate(depth);
    }

    fn advance(&mut self) {
        let mut frame = *self.top();
        let clause = self.clause();

        enum State {
            Satisfied,
            Guess(i32),
            Force(i32),
            Conflict,
        }

        let mut state = State::Conflict;

        for literal in &clause[frame.guess_index..] {
            frame.guess_index += 1;

            if self.is_assigned(*literal) {
                state = State::Satisfied;
                break;
            } else if self.is_assigned(-literal) {
                let (d, l) = self.resolution(-literal);

                if frame.resolve_depth < d {
                    frame.resolve_depth = d;
                    frame.resolve_literal = l;
                }
            } else {
                if frame.guess_index == clause.len() - 1 {
                    state = State::Force(*literal);
                } else {
                    state = State::Guess(*literal);
                }
                break;
            }
        }

        match state {
            State::Conflict => {
                if frame.resolve_depth == 0 {
                    self.status = SolverStatus::Unsat;
                    return;
                }

                self.pop_to(frame.resolve_depth);
                self.force(frame.resolve_literal);
            }
            State::Satisfied => {
                *self.top_mut() = frame;
            }
            State::Guess(literal) => {
                *self.top_mut() = frame;
                self.guess(literal);
            }
            State::Force(literal) => {
                *self.top_mut() = frame;
                self.force(literal);
            }
        }
    }

    pub fn status(&self) -> SolverStatus {
        self.status
    }

    pub fn finished(&self) -> bool {
        self.status() != SolverStatus::Unknown
    }

    pub fn step(&mut self) {
        if self.finished() {
            return;
        }

        self.push();
        self.advance();

        if self.depth() == 0 {
            self.status = SolverStatus::Unsat;
        } else if self.stack.len() == self.clauses.len() {
            self.status = SolverStatus::Sat;
        }
    }
}
