// Reference solutions:
// - https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day20/main.rs
//   doubly-linked list
// - https://github.com/schubart/AdventOfCode_2022_Rust/blob/master/day20/src/lib.rs
//   O(n^2) solution, finding the position of the element and removing and inserting it.
#![feature(binary_heap_into_iter_sorted)]

use color_eyre::Result;
use std::io::{self, Read};

mod naive;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = naive::parse(&input)?;
    println!("part 1: sum of three numbers: {}", task.part1());
    println!("part 1: sum after proper mixing: {}", task.part2());

    Ok(())
}
