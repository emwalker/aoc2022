use color_eyre::{self, eyre::eyre, Report, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    io,
    str::FromStr,
};

#[derive(Debug)]
struct Crate(String);

impl Display for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug)]
struct Stack {
    _label: usize,
    crates: VecDeque<Crate>,
}

impl Stack {
    fn pop(&mut self) -> Option<Crate> {
        self.crates.pop_front()
    }

    fn push(&mut self, c: Crate) {
        self.crates.push_front(c);
    }

    fn front(&self) -> Option<&Crate> {
        self.crates.front()
    }
}

#[derive(Debug)]
struct Move {
    count: u32,
    from: usize,
    to: usize,
}

impl FromStr for Move {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"move (?P<count>\d+) from (?P<from>\w+) to (?P<to>\w+)").unwrap();
        }

        if !RE.is_match(s) {
            return Err(eyre!("unknown move: {s}"));
        }

        if let Some(cap) = RE.captures(s) {
            match (cap.name("count"), cap.name("from"), cap.name("to")) {
                (Some(count), Some(from), Some(to)) => {
                    let count = count.as_str().parse::<u32>()?;
                    return Ok(Self {
                        count,
                        from: from.as_str().parse::<usize>()?,
                        to: to.as_str().parse::<usize>()?,
                    });
                }

                _ => return Err(eyre!("unknown move: {s}")),
            }
        };

        Err(eyre!("bad move: {s}"))
    }
}

#[derive(Debug, Default)]
struct StackBuilder {
    columns: HashMap<usize, VecDeque<Crate>>,
    stacks: HashMap<usize, Stack>,
}

impl StackBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn make_stacks(&mut self, line: &str) -> Result<bool> {
        if self.add_labels(line)? {
            return Ok(true);
        }

        if self.add_crates(line) {
            return Ok(true);
        }

        return Err(eyre!("failed to parse line: {line}"));
    }

    fn add_labels(&mut self, line: &str) -> Result<bool> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<label>\d+)").unwrap();
        }

        if !RE.is_match(line) {
            return Ok(false);
        }

        for (j, cap) in RE.captures_iter(line).enumerate() {
            let crates = self.columns.remove(&j).unwrap_or_default();
            if let Some(label) = cap.name("label") {
                let label = label.as_str().parse::<usize>()?;

                let stack = Stack {
                    crates,
                    _label: label,
                };

                self.stacks.insert(label, stack);
            }
        }

        Ok(true)
    }

    fn add_crates(&mut self, line: &str) -> bool {
        let mut curr = 0;
        let mut i = 0;

        while curr < line.len() {
            let width = (line.len() - curr).min(4);
            let field = &line[curr..(curr + width)];
            self.add_crate(i, field);
            curr += width;
            i += 1;
        }

        true
    }

    fn add_crate(&mut self, i: usize, s: &str) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\[(?P<label>\D)\]").unwrap();
        }

        let col: &mut VecDeque<_> = self.columns.entry(i).or_insert(VecDeque::new());

        if let Some(cap) = RE.captures_iter(s).next() {
            if let Some(label) = cap.name("label") {
                let c = Crate(label.as_str().to_owned());
                col.push_back(c);
            }
        }
    }

    #[allow(unused)]
    fn get(&self, index: usize) -> Option<&Stack> {
        self.stacks.get(&index)
    }

    #[allow(unused)]
    fn len(&self) -> usize {
        self.stacks.len()
    }

    fn finalize(self) -> Stacks {
        Stacks(self.stacks)
    }
}

#[derive(Debug)]
struct Stacks(HashMap<usize, Stack>);

impl Stacks {
    fn top_crates(&self) -> String {
        self.0
            .keys()
            .sorted()
            .flat_map(|label| self.0.get(label).unwrap().front())
            .join("")
    }

    fn move_crates(&mut self, line: &str) -> Result<bool> {
        let m = line.parse::<Move>()?;
        let inner = &mut self.0;

        for _ in 0..m.count {
            let from = inner
                .get_mut(&m.from)
                .unwrap()
                .pop()
                .ok_or(eyre!("nothing in stack {}", m.from))?;

            inner.get_mut(&m.to).unwrap().push(from);
        }

        Ok(true)
    }
}

struct Port(Box<dyn Iterator<Item = String>>);

impl Port {
    fn run(&mut self) -> Result<Stacks> {
        let mut builder = StackBuilder::new();
        let mut it = (&mut self.0).enumerate();

        for (i, line) in &mut it {
            if i > 0 && line.trim().is_empty() {
                break;
            }

            builder.make_stacks(&line)?;
        }

        let mut stacks = builder.finalize();

        for (_i, line) in it {
            stacks.move_crates(&line)?;
        }

        Ok(stacks)
    }
}

fn main() -> Result<()> {
    let it = io::stdin().lines().map(|line| line.unwrap());
    let mut port = Port(Box::new(it));
    let stacks = port.run()?;

    println!("top row: {}", stacks.top_crates());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_input() {
        let input = "    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

        let mut builder = StackBuilder::new();

        for line in input.lines() {
            if line.is_empty() {
                break;
            }

            if !builder.make_stacks(line).unwrap() {
                break;
            }
        }

        assert_eq!(builder.len(), 3);

        let stack = builder.get(1).unwrap();
        assert_eq!(stack.crates.len(), 2);

        let stack = builder.get(2).unwrap();
        assert_eq!(stack.crates.len(), 3);

        let stack = builder.get(3).unwrap();
        assert_eq!(stack.crates.len(), 1);

        let stacks = builder.finalize();
        assert_eq!(stacks.top_crates(), "NDP");
    }

    #[test]
    fn parse_move() {
        let m = "move 1 from 2 to 1".parse::<Move>().unwrap();
        assert_eq!(m.count, 1);
        assert_eq!(m.from, 2);
        assert_eq!(m.to, 1);
    }

    #[test]
    fn port() {
        let input = "    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

        let it = input.lines().map(str::to_owned);
        let mut port = Port(Box::new(it));
        let stacks = port.run().unwrap();
        assert_eq!(stacks.top_crates(), "CMZ");
    }
}
