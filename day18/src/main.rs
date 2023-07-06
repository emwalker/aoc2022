use color_eyre::{self, Result};
use day18::naive;
use std::io::{self, Read};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = naive::parse(&input)?;
    println!("part 1: surface area: {}", task.surface_area());

    Ok(())
}
