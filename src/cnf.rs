use std::collections::HashSet;

use std::cell::Cell;

pub struct Cnf {
    clauses: Vec<(usize, usize)>,
    literals: Vec<i32>,

    nbvars: Cell<Option<usize>>,
}

impl Cnf {
    pub fn add(&mut self, clause: &[i32]) -> usize {
        let pos = self.literals.len();
        let len = clause.len();
        let index = self.clauses.len();

        self.clauses.push((pos, len));
        self.literals.extend_from_slice(clause);

        self.nbvars.set(None);

        index
    }

    pub fn get(&self, index: usize) -> Option<&[i32]> {
        let (pos, len) = self.clauses.get(index)?;

        Some(&self.literals[*pos..*pos + *len])
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut [i32]> {
        let (pos, len) = self.clauses.get(index)?;

        Some(&mut self.literals[*pos..*pos + *len])
    }

    pub fn clauses(&self) -> impl Iterator<Item = &[i32]> {
        self.clauses
            .iter()
            .map(move |(pos, len)| &self.literals[*pos..*pos + *len])
    }

    pub fn nbclause(&self) -> usize {
        self.clauses.len()
    }

    pub fn nbvars(&self) -> usize {
        if let Some(nbvars) = self.nbvars.get() {
            return nbvars;
        }

        let nbvars = self
            .literals
            .iter()
            .map(|literal| literal.abs())
            .collect::<HashSet<i32>>()
            .len();

        self.nbvars.set(Some(nbvars));

        nbvars
    }
}
