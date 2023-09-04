use color_eyre::{eyre::eyre, Report, Result};
use itertools::Itertools;
use std::{
    fmt::Display,
    io::{self, Read},
    iter::Sum,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Snafu(i64);

impl Sum<Snafu> for Snafu {
    fn sum<I: Iterator<Item = Snafu>>(iter: I) -> Self {
        Self(iter.map(|Self(v)| v).sum())
    }
}

impl Display for Snafu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack: Vec<char> = vec![];
        let mut v = self.0;

        while v > 0 {
            let rem = v % 5;
            v /= 5;

            match rem {
                0 => stack.push('0'),
                1 => stack.push('1'),
                2 => stack.push('2'),

                3 => {
                    stack.push('=');
                    v += 1;
                }

                4 => {
                    stack.push('-');
                    v += 1;
                }

                _ => unreachable!(),
            };
        }

        write!(f, "{}", stack.iter().rev().join(""))
    }
}

impl FromStr for Snafu {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut v: i64 = 0;

        for c in s.trim().chars() {
            let digit = match c {
                '=' => -2,
                '-' => -1,
                '0' => 0,
                '1' => 1,
                '2' => 2,
                _ => return Err(eyre!("unknown digit: {}", c)),
            };

            v = (5 * v) + digit;
        }

        Ok(Self(v))
    }
}

struct Task {
    fuel: Vec<Snafu>,
}

impl Task {
    fn total_fuel(&self) -> Snafu {
        self.fuel.iter().copied().sum()
    }
}

fn parse(s: &str) -> Result<Task> {
    let fuel = s
        .trim()
        .lines()
        .map(|l| l.trim().parse::<Snafu>())
        .collect::<Result<Vec<Snafu>>>()?;

    Ok(Task { fuel })
}

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    let task = parse(&s)?;

    println!("total fuel: {}", task.total_fuel());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        let values = task.fuel.iter().map(|v| v.0).collect::<Vec<_>>();

        assert_eq!(values.len(), 13);
        assert_eq!(
            values,
            &[1747, 906, 198, 11, 201, 31, 1257, 32, 353, 107, 7, 3, 37],
        );
    }

    #[test]
    fn display() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        let values = task
            .fuel
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            &[
                "1=-0-2", "12111", "2=0=", "21", "2=01", "111", "20012", "112", "1=-1=", "1-12",
                "12", "1=", "122"
            ],
        )
    }

    #[test]
    fn values() {
        fn d(v: i64) -> String {
            format!("{}", Snafu(v))
        }

        assert_eq!(d(1), "1");
        assert_eq!(d(2), "2");
        assert_eq!(d(3), "1=");
        assert_eq!(d(4), "1-");
        assert_eq!(d(5), "10");
        assert_eq!(d(6), "11");
        assert_eq!(d(7), "12");
        assert_eq!(d(8), "2=");
        assert_eq!(d(9), "2-");
        assert_eq!(d(10), "20");
        assert_eq!(d(15), "1=0");
        assert_eq!(d(20), "1-0");
        assert_eq!(d(2022), "1=11-2");
        assert_eq!(d(12345), "1-0---0");
        assert_eq!(d(314159265), "1121-1110-1=0");
    }

    #[test]
    fn part1() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();

        let part1 = task.total_fuel();
        assert_eq!(part1, Snafu(4890));
        assert_eq!(format!("{}", part1), "2=-1=0");
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();

        let part1 = task.total_fuel();
        assert_eq!(part1, Snafu(34_191_676_204_851));
        assert_eq!(format!("{}", part1), "2-0-020-1==1021=--01");
    }
}
