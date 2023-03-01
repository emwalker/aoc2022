// Mostly taken from https://fasterthanli.me/series/advent-of-code-2022/part-16
use color_eyre::{self, Report, Result};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    io::{self, Read},
    str::FromStr,
};

mod parser;
use parser::{Name, Output, Valve};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Flow(u64);

type Connections = HashMap<Name, (Path, Flow)>;

type Path = Vec<(Name, Name)>;

#[derive(Debug)]
struct Network {
    valves: HashMap<Name, (Valve, Connections)>,
}

impl TryFrom<Output> for Network {
    type Error = Report;

    fn try_from(output: Output) -> std::result::Result<Self, Self::Error> {
        let mut network = Self {
            valves: output
                .iter()
                .map(|valve| (valve.name, (valve.to_owned(), Connections::default())))
                .collect(),
        };
        let names = network.valves.keys().copied().collect::<Vec<_>>();

        for name in names {
            // fill in the connections as needed
            let conns = network.connections(name);
            network.valves.get_mut(&name).unwrap().1 = conns;
        }

        Ok(network)
    }
}

impl Network {
    fn connections(&self, start: Name) -> Connections {
        let mut current: HashMap<Name, (Path, Flow)> = Default::default();

        {
            let valve = &self.valves[&start].0;
            current.insert(start, (vec![], Flow(valve.flow)));
        }

        let mut connections = current.clone();

        while !current.is_empty() {
            let mut next: HashMap<Name, (Path, Flow)> = Default::default();
            for (name, (path, _flow)) in current {
                for link in self.valves[&name].0.links.iter().copied() {
                    let valve = &self.valves[&link].0;
                    if let Entry::Vacant(e) = connections.entry(link) {
                        let conn_path: Path = path
                            .iter()
                            .copied()
                            .chain(std::iter::once((name, link)))
                            .collect();
                        let item = (conn_path.clone(), Flow(valve.flow));
                        e.insert(item.clone());
                        next.insert(link, item);
                    }
                }
            }
            current = next;
        }

        connections
    }
}

struct Move<'p> {
    pressure: u64,
    target: Name,
    path: &'p Path,
}

impl Move<'_> {
    fn cost(&self) -> u64 {
        1 + self.path.len() as u64
    }
}

#[derive(Clone, Debug)]
struct State<'n> {
    network: &'n Network,
    position: Name,
    max_turns: u64,
    turn: u64,
    pressure: u64,
    open_valves: HashSet<Name>,
}

impl State<'_> {
    fn apply(&self, mv: &Move) -> Self {
        let mut next = self.clone();
        next.position = mv.target;
        next.turn += mv.cost();
        next.pressure += mv.pressure;
        next.open_valves.insert(mv.target);
        next
    }

    fn moves(&self) -> impl Iterator<Item = Move> + '_ {
        let (_valve, connections) = &self.network.valves[&self.position];
        connections.iter().filter_map(|(name, (path, flow))| {
            if self.open_valves.contains(name) {
                return None;
            }

            if flow.0 == 0 {
                return None;
            }

            let travel_turns = path.len() as u64;
            let open_turns = 1_u64;
            let turns_spent_open = self.turns_left().checked_sub(travel_turns + open_turns)?;
            let pressure = flow.0 * turns_spent_open;
            Some(Move {
                pressure,
                target: *name,
                path,
            })
        })
    }

    fn turns_left(&self) -> u64 {
        self.max_turns - self.turn
    }

    fn apply_best_moves(&self) -> Self {
        let mut best_state = self.clone();

        for mv in self.moves() {
            let next = self.apply(&mv).apply_best_moves();
            if next.pressure > best_state.pressure {
                best_state = next;
            }
        }

        best_state
    }
}

struct Task {
    network: Network,
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let output = parser::parse(s)?;
        let network: Network = output.try_into()?;
        Ok(Self { network })
    }
}

impl Task {
    fn max_pressure_release(&self, max_turns: u64) -> Result<u64> {
        let state = State {
            network: &self.network,
            position: "AA".try_into()?,
            max_turns,
            turn: 0,
            pressure: 0,
            open_valves: HashSet::default(),
        };

        let state = state.apply_best_moves();
        Ok(state.pressure)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = input.parse::<Task>()?;
    println!(
        "pressure that can be released: {}",
        task.max_pressure_release(30)?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let output = parser::parse(input).unwrap();
        assert_eq!(output.0.len(), 10);
    }

    #[test]
    fn max_pressure_release() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.max_pressure_release(30).unwrap(), 1651);
    }
}
