// https://fasterthanli.me/series/advent-of-code-2022/part-17#part-2-rust
// Added for benchmarking purposes.
use color_eyre::{eyre::eyre, Report, Result};
use smallvec::{smallvec, SmallVec};

#[derive(Eq, PartialEq)]
enum Dir {
    Left,
    Right,
}

impl TryFrom<char> for Dir {
    type Error = Report;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '<' => Ok(Self::Left),
            '>' => Ok(Self::Right),
            _ => Err(eyre!(format!("unknown character: {}", value))),
        }
    }
}

pub struct Task {
    input: String,
}

impl Task {
    pub fn height_of_tower(&self, num_rocks: usize) -> usize {
        const CHAMBER_WIDTH: usize = 7;

        // this is our key type
        #[derive(Clone, PartialEq, Eq, Hash)]
        struct StateKey {
            // this could be a fixed-size array, but let's go easy for now
            relative_heights: [usize; CHAMBER_WIDTH],
            rock_index: usize,
            jet: usize,
        }

        // and this is our value type
        struct StateValue {
            highest: usize,
            total_rocks: usize,
        }

        // same as before
        let jets = self
            .input
            .trim()
            .chars()
            .map(|c| Dir::try_from(c).unwrap())
            .collect::<Vec<_>>();

        let rocks: Vec<SmallVec<[[usize; 2]; 16]>> = vec![
            smallvec![[2, 0], [3, 0], [4, 0], [5, 0]],
            smallvec![[2, 1], [3, 1], [3, 2], [3, 0], [4, 1]],
            smallvec![[2, 0], [3, 0], [4, 0], [4, 1], [4, 2]],
            smallvec![[2, 0], [2, 1], [2, 2], [2, 3]],
            smallvec![[2, 0], [3, 0], [2, 1], [3, 1]],
        ];

        // we've got a few more variables here
        let mut chamber = rustc_hash::FxHashSet::default();
        chamber.extend((0..7).map(|x| (x, 0)));

        let mut highest = 0;
        let mut jet: usize = 0;
        // XXX: 💀
        // the above is my original comment on this: this is extremely loosely
        // typed.  the key are 7 y-coordinates, which correspond to the height of
        // the rocks in the chamber, relative to the lowest one of them - see below
        // for a diagram.
        let mut states = rustc_hash::FxHashMap::<StateKey, StateValue>::default();
        let mut total_rocks = 0;
        let mut rock_index: usize = 0;
        let mut cycle_found = false;
        let mut height_gain_in_cycle = 0;
        let mut skipped_cycles = 0;
        let mut max_heights = [0usize; CHAMBER_WIDTH];

        // this is now a while loop, since we need to skip ahead using the cycles.
        while total_rocks < num_rocks {
            let mut rock = rocks[rock_index].clone();
            let adjustment = highest + 4;
            // set the rock's position to be at the current drop height
            for n in &mut rock {
                n[1] += adjustment;
            }
            let mut rest = false;
            while !rest {
                // move the rock left or right, as needed
                let mut new_rock = SmallVec::with_capacity(rock.len());
                if jets[jet] == Dir::Left {
                    // if we're moving left, update the rock to be one to the left
                    if rock[0][0] > 0 {
                        let mut good = true;
                        for n in &rock {
                            if chamber.contains(&(n[0] - 1, n[1])) {
                                good = false;
                                break;
                            }
                            new_rock.push([n[0] - 1, n[1]]);
                        }
                        if good {
                            rock = new_rock;
                        }
                    }
                } else {
                    // if we're moving right, update the rock to be one to the right
                    if rock.last().unwrap()[0] < 6 {
                        let mut good = true;
                        for n in &rock {
                            if chamber.contains(&(n[0] + 1, n[1])) {
                                good = false;
                                break;
                            }
                            new_rock.push([n[0] + 1, n[1]]);
                        }
                        if good {
                            rock = new_rock;
                        }
                    }
                }

                jet = (jet + 1) % jets.len();

                // move down if we can
                {
                    new_rock = smallvec![];
                    let mut good = true;
                    for n in &rock {
                        if chamber.contains(&(n[0], n[1] - 1)) {
                            for m in &rock {
                                // 👇
                                if max_heights[m[0]] < m[1] {
                                    max_heights[m[0]] = m[1];
                                }
                                chamber.insert((m[0], m[1]));
                            }
                            rest = true;
                            highest = rock
                                .iter()
                                .map(|n| n[1])
                                .chain([highest].into_iter())
                                .max()
                                .unwrap();
                            good = false;
                            break;
                        } else {
                            new_rock.push([n[0], n[1] - 1]);
                        }
                    }
                    if good {
                        rock = new_rock;
                    }
                }
            } // while !rest

            total_rocks += 1;

            // here's where cycle detection happens. we only try it if we haven't
            // already found a cycle earlier (which means we already skipped ahead,
            // which means we can just jump forward)
            if !cycle_found {
                let mut relative_heights = max_heights;
                let lowest = relative_heights.iter().copied().min().unwrap();

                for h in &mut relative_heights {
                    *h -= lowest;
                }

                let state_key = StateKey {
                    relative_heights,
                    rock_index,
                    jet,
                }; // lowest is `2`

                if let Some(state_value) = states.get(&state_key) {
                    // look mom! no more magic indexes! fields are named, and it
                    // doesn't come with any performance penalties (as python
                    // dictionaries might)
                    height_gain_in_cycle = highest - state_value.highest;
                    let rocks_in_cycle = total_rocks - state_value.total_rocks;
                    skipped_cycles = (num_rocks - total_rocks) / rocks_in_cycle;
                    total_rocks += skipped_cycles * rocks_in_cycle;
                    cycle_found = true;
                } else {
                    states.insert(
                        state_key,
                        StateValue {
                            highest,
                            total_rocks,
                        },
                    );
                }
            }

            rock_index = (rock_index + 1) % 5;
        }

        highest + (skipped_cycles * height_gain_in_cycle)
    }
}

pub fn parse(input: &str) -> Result<Task> {
    Ok(Task {
        input: input.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>\n";

    #[test]
    fn basics() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(3068, task.height_of_tower(2022));
    }

    #[test]
    fn part2() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(1_547_953_216_393, task.height_of_tower(1_000_000_000_000));
    }
}
