use std::io::BufRead;
use std::iter::FusedIterator;

/// Iterator to produce clauses from a DIMACS formatted `BufRead` stream.
pub struct Dimacs<R> {
    io: R,
    line: String,
}

impl<R> Dimacs<R>
where
    R: BufRead,
{
    /// Create a new `Dimacs<R>` structure with the given `BufRead` stream.
    pub fn new(io: R) -> Dimacs<R> {
        Dimacs {
            io,
            line: String::new(),
        }
    }
}

impl<R> FusedIterator for Dimacs<R> where R: BufRead {}

impl<R> Iterator for Dimacs<R>
where
    R: BufRead,
{
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // clear the string
            self.line.clear();

            // read a line into the string
            let bytes = self.io.read_line(&mut self.line).unwrap();

            // EOF condition
            if bytes == 0 {
                break;
            }

            // remove all whitespace at beginning and end of string
            let line = self.line.trim();

            // split the line into tokens, parse them as i32, drop the trailing 0, and the collect
            // into a Vec<i32> to return
            if !(line.is_empty() || line.starts_with('c') || line.starts_with('p')) {
                let clause: Vec<i32> = line
                    .split_whitespace()
                    .map(|token| token.parse::<i32>().unwrap())
                    .take_while(|literal| *literal != 0)
                    .collect();

                return Some(clause);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_01() {
        let string: Vec<u8> = r#"c
        c start with comments
        c
        c 
        p cnf 5 3
        1 -5 4 0
        -1 5 3 4 0
        -3 -4 0
        "#
        .bytes()
        .collect();

        let mut dimacs = Dimacs::new(&string[..]);

        assert_eq!(dimacs.next(), Some(vec![1, -5, 4]));
        assert_eq!(dimacs.next(), Some(vec![-1, 5, 3, 4]));
        assert_eq!(dimacs.next(), Some(vec![-3, -4]));
        assert_eq!(dimacs.next(), None);
    }
}
