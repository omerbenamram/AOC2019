use log::debug;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct Graph<V>
where
    V: Hash + Debug + Eq + Clone,
{
    adjacency_list: HashMap<V, HashSet<V>>,
}

impl<V> Graph<V>
where
    V: Hash + Debug + Eq + Clone,
{
    pub fn new() -> Self {
        Graph {
            adjacency_list: Default::default(),
        }
    }

    pub fn add_edge(&mut self, from: V, to: V) {
        self.adjacency_list
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    pub fn are_connected(&self, v1: &V, v2: &V) -> bool {
        self.adjacency_list
            .get(v1)
            .map(|edges| edges.contains(v2))
            .unwrap_or(false)
    }

    pub fn edges(&self, vertex: &V) -> Option<&HashSet<V>> {
        self.adjacency_list.get(vertex)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&V, &HashSet<V>)> {
        self.adjacency_list.iter()
    }

    pub fn into_iter(self) -> impl IntoIterator<Item = (V, HashSet<V>)> {
        self.adjacency_list.into_iter()
    }

    /// Returns a map of paths from vertex `start` to each other vertex in the graph.
    /// The key is the shortest path length.
    pub fn bfs(&self, start: V) -> HashMap<u32, HashSet<V>> {
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
