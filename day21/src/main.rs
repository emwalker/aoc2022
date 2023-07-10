// Reference solutions:
// - https://github.com/janiorca/advent-of-code-2022/blob/main/src/bin/aoc21.rs
//   Using a binary search
// - https://github.com/rrutkows/aoc2022/blob/main/src/d21/mod.rs
//   Fast, using an Op enum
// - https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day21/main.rs
//   Solving for humn.
use color_eyre::Result;
use day21::naive;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;

    let task = naive::parse(&s)?;
    println!("part 1: final number: {}", task.part1());
    println!("part 2: final number: {}", task.part2());

    Ok(())
}
