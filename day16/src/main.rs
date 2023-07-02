// This is a graph searching problem the speed of whose solution depends upon adequately pruning
// the search space.
//
// Let's try to get an optimal implementation using petgraph, even if it's not quite as fast as the
// best solution found in the Reddit comments.
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
    visited: u64,
    avoid: u64,
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

    fn branch<'a>(
        self,
        flows: &'a Flows,
        dists: &'a Distances,
    ) -> impl IntoIterator<Item = Self> + 'a {
        dists[self.pos]
            .iter()
            .enumerate()
            .filter(move |&(dest, _d)| self.can_visit(dest))
            .filter_map(move |(dest, d)| {
                let minutes_remaining = self.minutes_remaining.checked_sub(*d + 1)?;
                let pressure_released =
                    self.pressure_released + (minutes_remaining as u16 * flows[dest] as u16);

                Some(Self {
                    visited: self.visited | (1 << dest),
                    avoid: self.avoid,
                    pressure_released,
                    minutes_remaining,
                    pos: dest,
                })
            })
    }

    fn bound(self, flows: &Flows, sorted_indexes: &[usize]) -> u16 {
        let res = (0..=self.minutes_remaining)
            .rev()
            .step_by(2)
            .skip(1)
            .zip(
                sorted_indexes
                    .iter()
                    .filter(|&&i| self.can_visit(i))
                    .map(|&i| flows[i]),
            )
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

    for (i, valve) in valves.iter().enumerate() {
        for link in valve.links.iter() {
            let j = indexes[link];
            dists[i][j] = 1;
        }
    }

    for i in 0..n {
        dists[i][i] = 0;
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

fn branch_and_bound(
    flows: &Flows,
    sorted_indexes: &[usize],
    dists: &Distances,
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
        .branch(flows, dists)
        .into_iter()
        .map(|state| (state.bound(flows, sorted_indexes), state))
        .filter(|&(bound, _)| filter_bound(bound, *ans))
        .sorted_unstable_by_key(|(bound, _)| Reverse(*bound))
        .collect::<Vec<_>>();

    for (bound, branch) in pairs {
        if filter_bound(bound, *ans) {
            branch_and_bound(
                flows,
                sorted_indexes,
                dists,
                branch,
                max_for_visited,
                ans,
                filter_bound,
            );
        }
    }
}

struct Task {
    flows: Flows,
    distances: Distances,
    sorted_indexes: Vec<usize>,
    start: usize,
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let valves = parser::parse(s)?;
        let distances = shortest_distances(&valves);
        let flows = valves.iter().map(|valve| valve.flow).collect::<Flows>();

        // The number of valves must not exceed the size of our bit vectors
        assert!(valves.len() < 64);

        let indexes = valves
            .iter()
            .enumerate()
            .filter_map(|(i, valve)| {
                if valve.name == "AA" || valve.flow > 0 {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>();

        let sorted_indexes = indexes
            .iter()
            .sorted_unstable_by_key(|&&i| Reverse(flows[i]))
            .copied()
            .collect::<Vec<usize>>();

        let start = valves
            .iter()
            .enumerate()
            .find(|(_, valve)| valve.name == "AA")
            .unwrap()
            .0;

        Ok(Self {
            flows,
            distances,
            sorted_indexes,
            start,
        })
    }
}

impl Task {
    fn part1(&self) -> Result<u16> {
        let mut ans = 0;

        branch_and_bound(
            &self.flows,
            &self.sorted_indexes,
            &self.distances,
            State::new(self.start, 30),
            &mut [],
            &mut ans,
            |bound, best| bound > best,
        );

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
}
