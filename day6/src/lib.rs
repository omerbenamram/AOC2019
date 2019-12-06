use anyhow::{bail, Result};
use log::debug;
use std::collections::{HashMap, HashSet, VecDeque};

type Vertex = String;
const CENTER_OF_MASS: &str = "COM";

#[derive(Debug)]
struct Graph {
    adjacency_list: HashMap<Vertex, Vec<Vertex>>,
}

/// Graph of orbiting planets.
impl Graph {
    pub fn new() -> Self {
        Graph {
            adjacency_list: Default::default(),
        }
    }

    pub fn add_edge(&mut self, from: Vertex, to: Vertex) {
        self.adjacency_list.entry(from).or_insert(vec![]).push(to);
    }

    /// Returns a map of paths from vertex `start` to each other vertex in the graph.
    /// The key is the shortest path length.
    pub fn bfs(&self, start: Vertex) -> HashMap<u32, HashSet<Vertex>> {
        let mut queue = VecDeque::new();
        queue.push_back(&start);

        // Although the graph in this question should not contain loops, better safe than sorry.
        let mut visited = HashSet::new();
        visited.insert(&start);

        // Extra bookeeping to allow ourselves to keep track of depth
        // while getting away with using a queue for BFS (instead of "list of lists")
        let mut node_to_depth = HashMap::new();
        node_to_depth.insert(&start, 0);

        let mut layers = HashMap::new();
        let mut h = HashSet::new();
        h.insert(start.clone());
        layers.insert(0, h);

        while !queue.is_empty() {
            debug!("{:?}", &queue);
            let v = queue.pop_front().expect("Queue is not empty");

            if let Some(neighbors) = self.adjacency_list.get(v) {
                for neighbor in neighbors.iter() {
                    if !visited.contains(&neighbor) {
                        let parent_depth = node_to_depth.get(v).expect("parent must exist").clone();
                        let this_depth = parent_depth + 1;
                        node_to_depth.insert(neighbor, this_depth);
                        layers
                            .entry(this_depth)
                            .or_insert_with(HashSet::new)
                            .insert(neighbor.clone());

                        visited.insert(neighbor);
                        queue.push_back(neighbor)
                    }
                }
            }
        }

        layers
    }
}

pub fn part_1(input: &str) -> Result<u32> {
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
        // We invert the edges to be able to iterate them from `COM`.
        g.add_edge(edge[0].to_owned(), edge[1].to_owned());
    }

    let bfs = g.bfs(CENTER_OF_MASS.to_string());
    let mut total_orbits = 0;

    // we just count how many orbits are at each distance from `COM`
    for (k, v) in bfs {
        total_orbits += k * v.len() as u32
    }

    Ok(total_orbits)
}

pub fn part_2(input: &str) -> Result<u32> {
    let mut g = Graph::new();

    for line in input.lines() {
        let edge: Vec<&str> = line.trim().split(")").collect();
        if edge.len() != 2 {
            bail!(
                "Expected edge definition to be of pattern `A)B`, found `{}`",
                line
            );
        }

        // Orbital transfers don't care about direction
        g.add_edge(edge[0].to_owned(), edge[1].to_owned());
        g.add_edge(edge[1].to_owned(), edge[0].to_owned());
    }

    let bfs = g.bfs("YOU".to_string());

    for (len, vertexes) in bfs {
        if vertexes.contains(&"SAN".to_string()) {
            return Ok(len - 2);
        }
    }

    bail!("Path not found")
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

    #[test]
    pub fn test_part2() {
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
K)L
K)YOU
I)SAN";
        assert_eq!(part_2(input).unwrap(), 4);
    }
}
