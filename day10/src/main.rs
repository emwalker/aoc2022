use color_eyre::{self, eyre::eyre, Report, Result};
use itertools::Itertools;
use std::{
    fmt::{Debug, Display},
    io::{self, Read},
    str::FromStr,
};

#[derive(Clone, Debug)]
enum Instruction {
    Noop,
    AddX(i32),
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();

        if s == "noop" {
            return Ok(Self::Noop);
        }

        if let Some((t, count)) = s.split(' ').collect_tuple() {
            match (t, count.parse::<i32>()?) {
                ("addx", count) => return Ok(Self::AddX(count)),
                _ => return Err(eyre!("bad instruction: {s}")),
            }
        }

        Err(eyre!("bad instruction: {s}"))
    }
}

impl Instruction {
    fn cycles(&self) -> usize {
        match self {
            Self::Noop => 1,
            Self::AddX(..) => 2,
        }
    }

    fn value(&self) -> i32 {
        match self {
            Self::Noop => 0,
            Self::AddX(value) => *value,
        }
    }
}

#[derive(Clone, Debug)]
struct Program(Vec<Instruction>);

impl Program {
    fn parse(lines: &[String]) -> Result<Self> {
        let instructions: Vec<_> = lines
            .iter()
            .map(|l| l.parse::<Instruction>())
            .collect::<Result<Vec<Instruction>>>()?;
        if instructions.is_empty() {
            return Err(eyre!("expected at least one instruction"));
        }

        Ok(Self::new(instructions))
    }

    fn new(instructions: Vec<Instruction>) -> Self {
        Self(instructions)
    }

    fn readings(&self) -> ReadingIter {
        let ins = self.0.first().unwrap();
        let cycles_remaining = ins.cycles().checked_sub(1).unwrap();

        ReadingIter {
            program: self,
            cycle: 0,
            register: 1,
            cycles_remaining,
            i: 0,
        }
    }

    fn signal_strength(&mut self) -> impl Iterator<Item = i32> + '_ {
        self.readings()
            .skip(19)
            .step_by(40)
            .map(|r| r.signal_strength())
    }
}

#[derive(Debug)]
struct Reading {
    cycle: usize,
    register: i32,
}

impl Reading {
    fn signal_strength(&self) -> i32 {
        self.register * self.cycle as i32
    }
}

struct ReadingIter<'p> {
    cycles_remaining: usize,
    cycle: usize,
    i: usize,
    program: &'p Program,
    register: i32,
}

impl<'p> ReadingIter<'p> {
    fn instruction_at(&self, i: usize) -> &Instruction {
        &self.program.0[i % self.program.0.len()]
    }
}

impl<'p> Iterator for ReadingIter<'p> {
    type Item = Reading;

    fn next(&mut self) -> Option<Self::Item> {
        self.cycle += 1;

        let reading = Reading {
            cycle: self.cycle,
            register: self.register,
        };

        if self.cycles_remaining > 0 {
            self.cycles_remaining -= 1;
        } else {
            let ins = self.instruction_at(self.i);
            self.register += ins.value();
            self.i += 1;
            let ins = self.instruction_at(self.i);
            self.cycles_remaining = ins.cycles().checked_sub(1)?;
        }

        Some(reading)
    }
}

#[derive(Eq, PartialEq)]
struct CrtState(Vec<bool>);

const CRT_ROWS: usize = 6;
const CRT_COLS: usize = 40;

impl FromStr for CrtState {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut pixels = vec![false; CRT_ROWS * CRT_COLS];

        for (i, line) in s.lines().enumerate() {
            for (j, c) in line.trim().chars().enumerate() {
                let value = match c {
                    '#' => true,
                    '.' => false,
                    _ => return Err(eyre!("unexpected character at ({i}, {j}): {c}")),
                };

                if let Some(cell) = pixels.get_mut(i * CRT_COLS + j) {
                    *cell = value;
                } else {
                    return Err(eyre!("out of bounds access: ({i}, {j})"));
                }
            }
        }

        Ok(Self(pixels))
    }
}

impl Display for CrtState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = (0..CRT_ROWS)
            .map(move |i| {
                (0..CRT_COLS)
                    .map(|j| if self.0[i * CRT_COLS + j] { "#" } else { " " })
                    .join("")
            })
            .join("\n");

        f.write_str(&s)
    }
}

impl Debug for CrtState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\n{self}\n"))
    }
}

struct Task(Program);

impl Task {
    fn parse(lines: &[String]) -> Result<Self> {
        let program = Program::parse(lines)?;
        Ok(Self(program))
    }

    fn part1(&self) -> i32 {
        let mut p = self.0.clone();
        p.signal_strength().take(6).sum()
    }

    fn part2(&self) -> Result<CrtState> {
        let count = CRT_ROWS * CRT_COLS;
        let mut values = vec![false; count];

        for (i, reading) in self.0.readings().take(count).enumerate() {
            let value = reading.register;
            let v = (i % CRT_COLS) as i32;
            let pixel = (value - 1)..=(value + 1);
            values[i] = pixel.contains(&v);
        }

        Ok(CrtState(values))
    }
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let lines = input.lines().map(str::to_owned).collect_vec();

    let task = Task::parse(&lines)?;
    println!("part 1: {}", task.part1());
    println!("part 2:\n{}\n", task.part2()?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    fn program(input: &str) -> Program {
        let lines = input.lines().map(str::to_owned).collect_vec();
        Program::parse(&lines).unwrap()
    }

    #[test]
    fn simple_example() {
        let input = "\
        noop
        addx 3
        addx -5";
        let lines = input.lines().map(str::to_owned).collect_vec();
        let program = Program::parse(&lines).unwrap();
        let readings = program.readings().take(7).collect_vec();

        assert_eq!(readings[0].cycle, 1);
        assert_eq!(readings[0].register, 1);
        assert_eq!(readings[1].cycle, 2);
        assert_eq!(readings[1].register, 1);
        assert_eq!(readings[2].cycle, 3);
        assert_eq!(readings[2].register, 1);
        assert_eq!(readings[3].cycle, 4);
        assert_eq!(readings[3].register, 4);
        assert_eq!(readings[4].cycle, 5);
        assert_eq!(readings[4].register, 4);
        assert_eq!(readings[5].cycle, 6);
        assert_eq!(readings[5].register, -1);
    }

    #[test]
    fn register_value() {
        let p = program(include_str!("../data/example.txt"));
        let readings = p.readings().take(20).collect_vec();

        assert_eq!(readings[0].cycle, 1);
        assert_eq!(readings[0].register, 1);
        assert_eq!(readings[1].cycle, 2);
        assert_eq!(readings[1].register, 1);
        assert_eq!(readings[2].cycle, 3);
        assert_eq!(readings[2].register, 16);
        assert_eq!(readings[3].cycle, 4);
        assert_eq!(readings[3].register, 16);
        assert_eq!(readings[19].cycle, 20);
        assert_eq!(readings[19].register, 21);
    }

    #[test]
    fn signal_strength() {
        let mut p = program(include_str!("../data/example.txt"));

        assert_eq!(
            p.signal_strength().take(6).collect_vec(),
            vec![420, 1140, 1800, 2940, 2880, 3960]
        );
    }

    #[test]
    fn part1() {
        let task = Task(program(include_str!("../data/example.txt")));
        assert_eq!(task.part1(), 13140);
    }

    #[test]
    fn part1_with_data() {
        let task = Task(program(include_str!("../data/input.txt")));
        assert_eq!(task.part1(), 12740);
    }

    fn crt(input: &str) -> CrtState {
        input.parse::<CrtState>().unwrap()
    }

    #[test]
    fn part2() {
        let task = Task(program(include_str!("../data/example.txt")));

        let expected = crt("\
        ##..##..##..##..##..##..##..##..##..##..
        ###...###...###...###...###...###...###.
        ####....####....####....####....####....
        #####.....#####.....#####.....#####.....
        ######......######......######......####
        #######.......#######.......#######.....");

        assert_eq!(task.part2().unwrap(), expected);
    }

    #[test]
    fn part2_with_data() {
        let task = Task(program(include_str!("../data/input.txt")));

        // RBPARAGF
        let expected = crt("\
        ###..###..###...##..###...##...##..####.
        #..#.#..#.#..#.#..#.#..#.#..#.#..#.#....
        #..#.###..#..#.#..#.#..#.#..#.#....###..
        ###..#..#.###..####.###..####.#.##.#....
        #.#..#..#.#....#..#.#.#..#..#.#..#.#....
        #..#.###..#....#..#.#..#.#..#..###.#....");

        assert_eq!(task.part2().unwrap(), expected);
    }
}
