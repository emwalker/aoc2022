// Reference solutions
// - https://fasterthanli.me/series/advent-of-code-2022/part-17#part-2-rust
// - https://www.youtube.com/watch?v=QXTBseFzkW4 (Python)
//
// TODO:
// - Use criterion to get violin plots
// - Replace the rock-specific match statements with something more general
// - Model chamber as a hash of columns of heights (?)
// - Add cycle detection
use color_eyre::{self, Result};
use day17::naive;
use std::io::{self, Read};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = naive::parse(&input)?;
    println!(
        "part 1: height of tower, 2e03 steps: {}",
        task.height_of_tower(2022)
    );
    // println!(
    //     "part 2: height of tower, 1e12 steps: {}",
    //     task.height_of_tower(1000000000000)
    // );

    Ok(())
}
