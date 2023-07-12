#![feature(string_leak)]

// Reference solutions:
// - https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day21/main.rs
//   Solving for humn.
//
// Integer division makes it fiddly to use a bisect to solve part 2.  Some of the solutions that
// used a bisect either lucked out or were given input that didn't highlight the issue.
use color_eyre::Result;
use day21::solve;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    let s: &'static str = s.leak();

    let task = solve::parse(s)?;
    println!("part 1: final number: {}", task.part1());
    println!("part 2: final number: {}", task.part2());

    Ok(())
}
