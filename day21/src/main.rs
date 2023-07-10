use color_eyre::Result;
use day21::naive;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;

    let task = naive::parse(&s)?;
    println!("part 1: final number: {}", task.part1());

    Ok(())
}
