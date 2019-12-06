use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};

type Vertex = String;
const CENTER_OF_MASS: &str = "COM";

#[derive(Debug)]
struct Graph {
    adjacency_list: HashMap<Vertex, Vec<Vertex>>,
}

/// Directed graph of orbiting planets.
impl Graph {
    pub fn new() -> Self {
        Graph {
            adjacency_list: Default::default(),
        }
    }

    pub fn add_edge(&mut self, from: Vertex, to: Vertex) {
        self.adjacency_list.entry(from).or_insert(vec![]).push(to);
    }

    /// BFS to count all indirect orbits
    pub fn count_indirect_orbits(&self) -> i32 {
        let mut queue = VecDeque::new();
        queue.push_back(CENTER_OF_MASS);

        let mut visited = HashSet::new();
        visited.insert(CENTER_OF_MASS);

        let mut indirect_orbits = 0;
        let mut layers = HashMap::new();

        while !queue.is_empty() {
            println!("{:?}", &queue);
            depth += 1;
            let v = queue.pop_front().expect("Queue is not empty");

            if let Some(neighbors) = self.adjacency_list.get(v) {
                for neighbor in neighbors.iter() {
                    if !visited.contains(neighbor.as_str()) {
                        visited.insert(neighbor);
                        indirect_orbits += 1;
                        queue.push_back(neighbor)
                    }
                }
            }
        }
        dbg!(depth);

        indirect_orbits
    }

    /// By the assumption that
    pub fn count_direct_orbits(&self) -> i32 {
        0
    }
}

pub fn part_1(input: &str) -> Result<i32> {
    let mut g = Graph::new();

    for line in input.lines() {
        let edge: Vec<&str> = line.trim().split(")").collect();
        if edge.len() != 2 {
            bail!(
                "Expected edge definition to be of pattern `A)B`, found `{}`",
                line
            );
        }

        // B orbits A translates to
        // B --> A
        g.add_edge(edge[0].to_owned(), edge[1].to_owned());
    }

    dbg!(&g);
    let indirect_orbits = g.count_indirect_orbits();

    Ok(indirect_orbits)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_part1() {
        let input = "\
        COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L";
        assert_eq!(part_1(input).unwrap(), 42);
    }
}
