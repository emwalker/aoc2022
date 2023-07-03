// This is a graph searching problem the speed of whose solution depends upon adequately pruning
// the search space.
//
// Part 1
//
//   - Approach: Use Floyd-Warshall to compute the minimum distance/max flow between each pair of
//     vertices.  Trim the vertices at which the flow rate is zero.  Use branch-and-bound to solve
//     the traveling salesman problem (?).
//   - Approach: Use Dijkstra to build up a matrix of shortest distances between valves with non-
//     zero flows.  Depth-first search of all paths that we have time to visit.
//
// Part 2
//
//   - Approach: Compute the best pressure for each set of visited valves over 26 minutes.
//     Combine the two solutions that visit a disjoint set of valves.
//   - Use bitmask to compute mutually exclusive sets of valves
//
// Reference solutions
//
//   - https://www.reddit.com/r/adventofcode/comments/zn6k1l/comment/j0pewzt/. Rust, 2ms.
//   - https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day16/main.rs
//   - https://www.reddit.com/r/adventofcode/comments/zn6k1l/comment/j1piehq/. Rust, 8ms.
//   - https://github.com/orlp/aoc2022/blob/master/src/bin/day16.rs
//   - https://www.reddit.com/r/adventofcode/comments/zn6k1l/comment/j0gmocd/. Rust, dp, 180ms.
//   - https://www.reddit.com/r/adventofcode/comments/zn6k1l/comment/j0oo5a9/. Rust.  Uses a bitmask
//     to compute mutually exclusive paths of values for part 2.
//   - https://www.reddit.com/r/adventofcode/comments/zn6k1l/comment/j0k26sn/. Rust. Use a dfs to
//     find distances for part 1, recursive dfs to find optimal path, pruning the search if a path
//     overlaps with the valves in a list provided)
//   - https://www.reddit.com/r/adventofcode/comments/zn6k1l/comment/j0rsxjc/ (Rust, use Dijkstra
//     to compute the shortest distances between each valve.  Filter out the valves with zero flow.
//     Depth first search of all of the paths we have time to visit.)
//
use color_eyre::{self, Report, Result};
use itertools::Itertools;
use std::{
    cmp::Reverse,
    collections::HashMap,
    io::{self, Read},
    str::FromStr,
};

mod parser;
use parser::Valves;

type Distances = Vec<Vec<u8>>;
type Flows = Vec<u8>;

#[derive(Default, Debug, Clone, Copy)]
struct State {
    visited: u16,
    avoid: u16,
    pressure_released: u16,
    minutes_remaining: u8,
    pos: usize,
}

impl State {
    fn new(pos: usize, minutes_remaining: u8) -> Self {
        Self {
            visited: 0,
            avoid: 1 << pos,
            pressure_released: 0,
            minutes_remaining,
            pos,
        }
    }

    fn can_visit(self, i: usize) -> bool {
        (self.visited | self.avoid) & (1 << i) == 0
    }

    fn branch(self, net: &Network) -> impl IntoIterator<Item = Self> + '_ {
        net.dists[self.pos]
            .iter()
            .enumerate()
            .filter(move |&(dest, _d)| self.can_visit(dest))
            .filter_map(move |(dest, d)| {
                let minutes_remaining = self.minutes_remaining.checked_sub(*d + 1)?;
                let pressure_released =
                    self.pressure_released + (minutes_remaining as u16 * net.flows[dest] as u16);

                Some(Self {
                    visited: self.visited | (1 << dest),
                    avoid: self.avoid,
                    pressure_released,
                    minutes_remaining,
                    pos: dest,
                })
            })
    }

    fn bound(self, net: &Network) -> u16 {
        let sorted_flows = net
            .sorted_indexes
            .iter()
            .filter(|&&i| self.can_visit(i))
            .map(|&i| net.flows[i]);

        // TODO: Figure out what is going on here.
        let res = (0..=self.minutes_remaining)
            .rev()
            .step_by(2)
            .skip(1)
            .zip(sorted_flows)
            .map(|(minutes, flow)| minutes as u16 * flow as u16)
            .sum::<u16>();

        res + self.pressure_released
    }
}

// Use the Floyd-Warshall algorithm to compute minimum distances between each pair of vertices.
fn shortest_distances(valves: &Valves) -> Distances {
    let indexes = valves
        .iter()
        .enumerate()
        .map(|(i, valve)| (&valve.name, i))
        .collect::<HashMap<&String, _>>();
    let n = valves.len();
    let mut dists = vec![vec![u8::MAX; n]; n];

    // Valves are one step away from their neighbors
    for (i, valve) in valves.iter().enumerate() {
        for link in valve.links.iter() {
            let j = indexes[link];
            dists[i][j] = 1;
        }
    }

    // Valves are zero steps away from themselves
    for (i, row) in dists.iter_mut().enumerate() {
        row[i] = 0;
    }

    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                let d = dists[i][k].saturating_add(dists[k][j]);
                if dists[i][j] > d {
                    dists[i][j] = d;
                }
            }
        }
    }

    dists
}

struct Network {
    flows: Flows,
    sorted_indexes: Vec<usize>,
    dists: Distances,
}

impl Network {
    fn branch_and_bound(
        &self,
        state: State,
        max_for_visited: &mut [u16],
        ans: &mut u16,
        filter_bound: impl Fn(u16, u16) -> bool + Copy,
    ) {
        if let Some(curr_max) = max_for_visited.get_mut(state.visited as usize) {
            *curr_max = state.pressure_released.max(*curr_max);
        }
        *ans = state.pressure_released.max(*ans);

        let pairs = state
            .branch(self)
            .into_iter()
            .map(|state| (state.bound(self), state))
            .filter(|&(bound, _)| filter_bound(bound, *ans))
            .sorted_unstable_by_key(|(bound, _)| Reverse(*bound))
            .collect::<Vec<_>>();

        for (bound, branch) in pairs {
            if filter_bound(bound, *ans) {
                self.branch_and_bound(branch, max_for_visited, ans, filter_bound);
            }
        }
    }
}

struct Task {
    network: Network,
    start: usize,
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let valves = parser::parse(s)?;
        let full_dists = shortest_distances(&valves);

        let interesting_subset = valves
            .iter()
            .enumerate()
            .filter(|&(_i, valve)| valve.name == "AA" || valve.flow > 0)
            .map(|(i, _valve)| i)
            .collect::<Vec<_>>();

        // The number of valves must not exceed the size of our u16 bit vectors.
        // Using u32 bit vectors causes the search space to be too large.
        assert!(interesting_subset.len() <= 16);

        let flows = interesting_subset
            .iter()
            .map(|&i| valves[i].flow)
            .collect::<Vec<_>>();

        let dists = interesting_subset
            .iter()
            .map(|&i| {
                interesting_subset
                    .iter()
                    .map(|&j| full_dists[i][j])
                    .collect()
            })
            .collect::<Vec<_>>();

        let sorted_indexes = flows
            .iter()
            .enumerate()
            .sorted_unstable_by_key(|&(_, &flow)| Reverse(flow))
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        let start = interesting_subset
            .iter()
            .position(|&i| valves[i].name == "AA")
            .expect("an AA valve");

        let network = Network {
            flows,
            dists,
            sorted_indexes,
        };

        Ok(Self { network, start })
    }
}

impl Task {
    fn part1(&self) -> Result<u16> {
        let mut ans = 0;

        self.network.branch_and_bound(
            State::new(self.start, 30),
            &mut [],
            &mut ans,
            |bound, best| bound > best,
        );

        Ok(ans)
    }

    fn part2(&self) -> Result<u16> {
        let mut max_for_visited = vec![0; u16::MAX as usize];

        self.network.branch_and_bound(
            State::new(self.start, 26),
            &mut max_for_visited,
            &mut 0,
            // TODO: Figure out why we can't use the previous filter function
            |bound, best| bound > (best * 3 / 4),
        );

        let sorted_max = max_for_visited
            .into_iter()
            .enumerate()
            .filter(|&(_, max)| max > 0)
            .map(|(i, max)| (i as u16, max))
            .sorted_unstable_by_key(|&(_, max)| Reverse(max))
            .collect::<Vec<_>>();

        let mut ans = 0;

        for (i, &(elf_visited, elf_max)) in sorted_max.iter().enumerate() {
            for &(ele_visited, ele_max) in &sorted_max[i + 1..] {
                let score = elf_max + ele_max;
                if score <= ans {
                    break;
                }

                if elf_visited & ele_visited == 0 {
                    ans = score;
                    break;
                }
            }
        }

        Ok(ans)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = input.parse::<Task>()?;
    println!(
        "part 1: max pressure that can be released: {}",
        task.part1()?
    );

    println!(
        "part 2: max pressure with the help of an elephant: {}",
        task.part2()?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let valves = parser::parse(input).unwrap();
        assert_eq!(valves.len(), 10);
    }

    #[test]
    fn distances() {
        let input = include_str!("../data/example.txt");
        let valves = parser::parse(input).unwrap();
        let dists = shortest_distances(&valves);
        assert_eq!(dists[0][0], 0);
        assert_eq!(dists[0][1], 1);
        assert_eq!(dists[2][5], 3);
    }

    #[test]
    fn max_pressure_release() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.part1().unwrap(), 1651);
    }

    #[test]
    fn with_elephant() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.part2().unwrap(), 1707);
    }

    #[test]
    fn input_values() {
        let input = include_str!("../data/input.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.part1().unwrap(), 2359);
        assert_eq!(task.part2().unwrap(), 2999);
    }
}
