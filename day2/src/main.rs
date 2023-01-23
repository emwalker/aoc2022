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
    their_move: Move,
    // Part 1
    our_move: Move,
    // Part 2
    desired_outcome: Outcome,
}

impl FromStr for Round {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let moves: Vec<_> = s.split(' ').collect();
        if moves.len() != 2 {
            return Err(format!("unexpected input: {:?}", moves));
        }

        let their_move = match moves[0] {
            "A" => Move::Rock,
            "B" => Move::Paper,
            "C" => Move::Scissors,
            _ => return Err(format!("invalid move: {}", moves[0])),
        };

        let (our_move, desired_outcome) = match moves[1] {
            "X" => (Move::Rock, Outcome::TheirWin),
            "Y" => (Move::Paper, Outcome::Draw),
            "Z" => (Move::Scissors, Outcome::OurWin),
            _ => return Err(format!("invalid move: {}", moves[1])),
        };

        Ok(Round {
            their_move,
            our_move,
            desired_outcome,
        })
    }
}

impl Round {
    fn part1_score(&self) -> i32 {
        (self.part1_result() as i32) + (self.our_move as i32)
    }

    fn part2_score(&self) -> i32 {
        // TODO: generalize
        let our_move = match (self.their_move, self.desired_outcome) {
            (Move::Rock, Outcome::TheirWin) => Move::Scissors,
            (Move::Rock, Outcome::Draw) => Move::Rock,
            (Move::Rock, Outcome::OurWin) => Move::Paper,

            (Move::Paper, Outcome::TheirWin) => Move::Rock,
            (Move::Paper, Outcome::Draw) => Move::Paper,
            (Move::Paper, Outcome::OurWin) => Move::Scissors,

            (Move::Scissors, Outcome::TheirWin) => Move::Paper,
            (Move::Scissors, Outcome::Draw) => Move::Scissors,
            (Move::Scissors, Outcome::OurWin) => Move::Rock,
        };

        (self.desired_outcome as i32) + (our_move as i32)
    }

    fn part1_result(&self) -> Outcome {
        if self.their_move == self.our_move {
            return Outcome::Draw;
        }

        match (&self.their_move, &self.our_move) {
            (Move::Rock, Move::Paper) => Outcome::OurWin,
            (Move::Paper, Move::Scissors) => Outcome::OurWin,
            (Move::Scissors, Move::Rock) => Outcome::OurWin,
            _ => Outcome::TheirWin,
        }
    }
}

struct Runner {
    pub part1_scores: Vec<i32>,
    pub part2_scores: Vec<i32>,
}

impl Runner {
    fn parse(input: &str) -> Result<Self> {
        let lines: Vec<&str> = input.lines().map(|l| l.trim()).collect();
        let mut part1_scores = vec![];
        let mut part2_scores = vec![];

        for line in lines.into_iter() {
            let round = line.parse::<Round>().unwrap();
            part1_scores.push(round.part1_score());
            part2_scores.push(round.part2_score());
        }

        Ok(Self {
            part1_scores,
            part2_scores,
        })
    }

    fn part1_total(&self) -> i32 {
        self.part1_scores.iter().sum()
    }

    fn part2_total(&self) -> i32 {
        self.part2_scores.iter().sum()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;
    let runner = Runner::parse(&input)?;
    println!("part 1 total: {}", runner.part1_total());
    println!("part 2 total: {}", runner.part2_total());
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
        assert_eq!(runner.part1_scores.len(), 3);
        assert_eq!(runner.part1_scores[0], 8);
        assert_eq!(runner.part1_scores[1], 1);
        assert_eq!(runner.part1_scores[2], 6);

        assert_eq!(runner.part1_total(), 15);
        assert_eq!(runner.part2_total(), 12);
    }
}
