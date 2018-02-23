use Error::*;

#[derive(Debug)]
enum Error {
    InvalidChar { actual: char },
    OutBound,
    UnexpectedChar { actual: char },
    UndefinedSymbol { actual: char },
}

struct Machine<'a> {
    prog: &'a str,
    funcmap: [Option<&'a str>; 28],
}

impl <'a> Machine<'a> {
    fn eval(&mut self, args: &[isize]) -> Result<isize, Error> {
        self.skip();

        let c = self.peek(0)?;
        match c {
            'a'...'z' => {
                self.seek(1);
                Ok(*args.get((c as usize) - ('a' as usize)).ok_or_else(
                    || UndefinedSymbol { actual: c })?)
            },
            'P' => {
                self.seek(1);
                self.expect('(')?;
                self.seek(1);
                self.skip();
                let v = self.eval(args)?;
                self.skip();
                self.expect(')')?;
                self.seek(1);
                println!("{}", v);
                Ok(v)
            },
            'A'...'Z' if self.peek(1)? == '[' => {
                self.seek(2);
                let n = self.prog.find(|c: char| c == ']').ok_or(OutBound)?;
                self.funcmap[(c as usize) - ('A' as usize)] = Some(&self.prog[..n]);
                self.seek(n + 1);
                self.eval(args)
            },
            'A'...'Z' if self.peek(1)? == '(' => {
                self.seek(2);

                let f = self.funcmap[(c as usize) - ('A' as usize)]
                    .ok_or_else(|| UndefinedSymbol { actual: c })?;

                let mut new_args = [0; 28];
                let mut i = 0;
                self.skip();
                while self.peek(0)? != ')' {
                    new_args[i] = self.eval(args)?;
                    i += 1;
                    self.skip();
                }
                self.expect(')')?;
                self.seek(1);

                let old_prog = self.prog;
                self.prog = f;
                let mut r = self.eval(&new_args)?;
                self.skip();
                while !self.prog.is_empty() {
                    r = self.eval(&new_args)?;
                    self.skip();
                }
                self.prog = old_prog;
                Ok(r)
            },
            '+' | '-' | '*' | '/' => {
                self.seek(1);
                let x = self.eval(args)?;
                let y = self.eval(args)?;
                Ok(match c {
                    '+' => x + y,
                    '-' => x - y,
                    '*' => x * y,
                    '/' => x / y,
                    _ => unreachable!(),
                })
            },
            '0'...'9' => {
                let old_prog = self.prog;
                while !self.prog.is_empty() && self.peek(0)?.is_digit(10) {
                    self.seek(1);
                }
                Ok(old_prog[..(old_prog.len() - self.prog.len())].parse().unwrap())
            },
            _ => Err(InvalidChar { actual: c }),
        }
    }

    fn peek(&self, n: usize) -> Result<char, Error> {
        self.prog[n..].chars().next().ok_or(OutBound)
    }

    fn expect(&self, e: char) -> Result<(), Error> {
        let c = self.peek(0)?;
        if c != e {
            Err(UnexpectedChar { actual: c  })
        } else {
            Ok(())
        }
    }

    fn seek(&mut self, n: usize) {
        self.prog = &self.prog[n..];
    }

    fn skip(&mut self) {
        self.prog = self.prog.trim_left();
    }
}

fn main() {
    let mut m = Machine {
        prog: &std::env::args().nth(1).unwrap_or_else(|| panic!("Missing argument")),
        funcmap: [None; 28],
    };
    while !m.prog.is_empty() {
        match m.eval(&[]) {
            Ok(v) => println!("{}", v),
            Err(e) => panic!("{:?}", e),
        }
        m.skip();
    }
}
