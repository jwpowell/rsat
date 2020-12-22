use std::io::BufRead;

pub struct Dimacs<R> {
    io: R,

    line: String,

    var_count: usize,
    clause_count: usize,
}

impl<R> Dimacs<R>
where
    R: BufRead,
{
    pub fn new(io: R) -> Dimacs<R> {
        let mut dimacs = Dimacs {
            io,
            line: String::new(),
            var_count: 0,
            clause_count: 0,
        };

        loop {
            dimacs.line.clear();
            dimacs.io.read_line(&mut dimacs.line);
            if dimacs.line.starts_with('p') {
                let mut iter = dimacs.line.split_whitespace();

                assert_eq!(iter.next(), Some("p"), "line must start with p");
                assert_eq!(iter.next(), Some("cnf"), "line must specify cnf");

                dimacs.var_count = iter.next().unwrap().parse::<usize>().unwrap();
                dimacs.clause_count = iter.next().unwrap().parse::<usize>().unwrap();
                break;
            }
        }

        dimacs
    }

    pub fn next(&mut self, literals: &mut Vec<i32>) -> bool {
        self.line.clear();
        let count = self.io.read_line(&mut self.line).unwrap();

        if count == 0 {
            return false;
        }

        literals.clear();
        literals.extend(
            self.line
                .split_whitespace()
                .map(|tok| tok.parse::<i32>().unwrap())
                .take_while(|n| *n != 0),
        );

        true
    }

    pub fn var_count(&self) -> usize {
        self.var_count
    }

    pub fn clause_count(&self) -> usize {
        self.clause_count
    }
}
