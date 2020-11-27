use std::io::BufRead;

pub struct Dimacs<R> {
    io: R,
    line: String,
}

impl<R> Dimacs<R>
where
    R: BufRead,
{
    pub fn new(io: R) -> Dimacs<R> {
        Dimacs {
            io,
            line: String::new(),
        }
    }
}

impl<R> Iterator for Dimacs<R>
where
    R: BufRead,
{
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.line.clear();
            let bytes = self.io.read_line(&mut self.line).unwrap();

            if bytes == 0 {
                break;
            }

            let line = self.line.trim();

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
