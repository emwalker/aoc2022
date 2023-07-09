// Reference solutions:
// - https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day19/main.rs
//   Rust, branch and bound
// - https://github.com/orlp/aoc2022/blob/master/src/bin/day19.rs
//   Rust, branch and bound
// - https://github.com/aaronblohowiak/advent-of-code-2022/blob/main/nineteen/src/main.rs
//   dfs solution
// - https://gist.github.com/Stevie-O/34e35fb1e9361b38105201c63eb47d25
//   another branch and bound solution
// - https://www.reddit.com/r/adventofcode/comments/zpihwi/comment/j0w89n9/
//   another Rust solution
use color_eyre::Result;
use day19::branch1;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = branch1::parse(&input)?;
    println!("part 1: quality level: {}", task.total_quality_level());
    println!("part 2: product of first three: {}", task.first_three());

    Ok(())
}
