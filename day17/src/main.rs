// Reference solutions
// - https://fasterthanli.me/series/advent-of-code-2022/part-17#part-2-rust
// - https://www.youtube.com/watch?v=QXTBseFzkW4 (Python)
use color_eyre::{self, Result};
use day17::cycles;
use std::io::{self, Read};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // The final and fastest solution that can complete both parts
    let task = cycles::parse(&input)?;
    println!(
        "part 1: height after 2e03 steps: {}",
        task.height_of_tower(2022)
    );
    println!(
        "part 2: height after 1e12 steps: {}",
        task.height_of_tower(1_000_000_000_000)
    );

    Ok(())
}
