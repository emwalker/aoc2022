use color_eyre::{self, Result};
use std::{
    io::{self, Read},
    str::FromStr,
};

#[derive(Clone, Copy, Debug)]
enum Outcome {
    TheirWin = 0,
    Draw = 3,
    OurWin = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Move {
    Rock = 1,
    Paper = 2,
    Scissors = 3,
}

#[derive(Debug)]
struct Round {
    theirs: Move,
    ours: Move,
}

impl FromStr for Round {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let moves: Vec<_> = s.split(' ').collect();
        if moves.len() != 2 {
            return Err(format!("unexpected input: {:?}", moves));
        }

        let theirs = match moves[0] {
            "A" => Move::Rock,
            "B" => Move::Paper,
            "C" => Move::Scissors,
            _ => return Err(format!("invalid move: {}", moves[0])),
        };

        let ours = match moves[1] {
            "X" => Move::Rock,
            "Y" => Move::Paper,
            "Z" => Move::Scissors,
            _ => return Err(format!("invalid move: {}", moves[1])),
        };

        Ok(Round { theirs, ours })
    }
}

impl Round {
    fn our_score(&self) -> i32 {
        (self.our_result() as i32) + (self.ours as i32)
    }

    fn our_result(&self) -> Outcome {
        if self.theirs == self.ours {
            return Outcome::Draw;
        }

        match (&self.theirs, &self.ours) {
            (Move::Rock, Move::Paper) => Outcome::OurWin,
            (Move::Paper, Move::Scissors) => Outcome::OurWin,
            (Move::Scissors, Move::Rock) => Outcome::OurWin,
            _ => Outcome::TheirWin,
        }
    }
}

struct Runner {
    pub scores: Vec<i32>,
}

impl Runner {
    fn parse(input: &str) -> Result<Self> {
        let lines: Vec<&str> = input.lines().map(|l| l.trim()).collect();
        let mut scores = vec![];

        for line in lines.into_iter() {
            let round = line.parse::<Round>().unwrap();
            scores.push(round.our_score());
        }

        Ok(Self { scores })
    }

    fn total_score(&self) -> i32 {
        self.scores.iter().sum()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;
    let runner = Runner::parse(&input)?;
    println!("total score: {}", runner.total_score());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_input() {
        let input = "A Y
        B X
        C Z";

        let runner = Runner::parse(input).unwrap();
        assert_eq!(runner.scores.len(), 3);
        assert_eq!(runner.scores[0], 8);
        assert_eq!(runner.scores[1], 1);
        assert_eq!(runner.scores[2], 6);
        assert_eq!(runner.total_score(), 15);
    }
}
