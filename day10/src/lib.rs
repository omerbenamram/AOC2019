use anyhow::{bail, Context, Result};
use aoc_graph::Graph;
use itertools::{all, Itertools};
use log::debug;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

type Coord = (i32, i32);

fn parse_input(input: &str) -> Vec<Coord> {
    input
        .trim()
        .lines()
        .enumerate()
        .map(|(y, line)| {
            line.chars()
                .enumerate()
                .filter_map(|(x, c)| match c {
                    '#' => Some((x as i32, y as i32)),
                    _ => None,
                })
                .collect::<Vec<Coord>>()
        })
        .flatten()
        .collect()
}

fn distance(a: Coord, b: Coord) -> f32 {
    (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f32).sqrt()
}

fn angle(a: Coord, b: Coord) -> f32 {
    let dx = (a.0 - b.0) as f32;
    let dy = (a.1 - b.1) as f32;
    dx.atan2(dy)
}

fn angle_abs(a: Coord, b: Coord) -> f32 {
    let dx = (a.0 - b.0) as f32;
    let dy = (a.1 - b.1) as f32;
    let atan = dx.atan2(dy);

    if atan > 0.0 {
        (2.0 * std::f32::consts::PI - atan)
    } else {
        atan * -1.0
    }
}

pub fn part_1(input: &str) -> Result<(Coord, usize)> {
    let astroids = parse_input(input);

    if astroids.is_empty() {
        bail!("Input is empty.");
    }

    let mut edges_to_slope = HashMap::new();
    let mut graph = Graph::new();

    // Build slope graph
    for astroid in astroids.iter().cloned() {
        for another in astroids.iter().cloned() {
            if astroid == another {
                continue;
            }
            if graph.are_connected(&astroid, &another) {
                continue;
            }

            let angle = angle(astroid, another);

            graph.add_edge(astroid, another);
            edges_to_slope.insert((astroid, another), angle);

            graph.add_edge(another, astroid);
            edges_to_slope.insert((another, astroid), std::f32::consts::PI - angle);
        }
    }

    // For each two lines with the same slope, only the two
    let mut visibility_graph = Graph::new();

    for (vertex, edges) in graph.into_iter() {
        for (slope, edges) in &edges
            .into_iter()
            .sorted_by(|edge_1, edge_2| {
                edges_to_slope
                    .get(&(vertex, *edge_1))
                    .unwrap()
                    .partial_cmp(edges_to_slope.get(&(vertex, *edge_2)).unwrap())
                    .unwrap_or(Ordering::Equal)
            })
            .group_by(|&another| edges_to_slope.get(&(vertex, another)).expect("inserted"))
        {
            let edges = edges.collect::<Vec<Coord>>();

            let closest = edges
                .into_iter()
                .map(|another| (another, distance(vertex, another)))
                .min_by(|edge_1, edge_2| edge_1.1.partial_cmp(&edge_2.1).unwrap_or(Ordering::Less))
                .context("A minimum should exist")?;

            visibility_graph.add_edge(vertex, closest.0);
            visibility_graph.add_edge(closest.0, vertex);
        }
    }

    let max = visibility_graph
        .iter()
        .max_by_key(|(v, edges)| edges.len())
        .context("Inconclusive maximum")?;

    Ok((*max.0, max.1.len()))
}

#[derive(Eq, Debug, Clone)]
struct AstroidWithDistance {
    coord: Coord,
    distance: i32,
}

impl PartialEq for AstroidWithDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl PartialOrd for AstroidWithDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Ord for AstroidWithDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(Ordering::Equal)
    }
}

pub fn part_2(input: &str) -> Result<(Coord, usize)> {
    let astroids = parse_input(input);

    if astroids.is_empty() {
        bail!("Input is empty.");
    }

    let (start, _) = part_1(input)?;

    // {Angle -> [Vertex Sorted By Distance]}
    let mut laser_queue = HashMap::new();
    // Cannot use angle as f32 key, but we still need to know the order..
    let mut all_angles = Vec::new();

    // Build slope graph
    for another in astroids.iter().cloned() {
        if another == start {
            continue;
        }
        let angle = angle_abs(start, another);
        let distance = distance(start, another) * 10000.0;
        let ast = AstroidWithDistance {
            coord: another,
            distance: distance.round() as i32,
        };

        all_angles.push(angle);

        laser_queue
            .entry(format!("{}", angle))
            .or_insert_with(BinaryHeap::new)
            .push(std::cmp::Reverse(ast));
    }

    all_angles.sort_by(|f1, f2| f1.partial_cmp(f2).unwrap_or(Ordering::Equal));

    let keys: Vec<String> = all_angles
        .iter()
        .map(|f| format!("{}", f))
        .dedup()
        .collect();

    let mut keys_iter = keys.iter().cycle();
    let mut number_of_astroids_destroyed = 0;
    let total_astroids = astroids.len();
    let mut last_destroyed = None;

    while (number_of_astroids_destroyed < 200) && (number_of_astroids_destroyed <= total_astroids) {
        debug!("{} -> {:?}", number_of_astroids_destroyed, last_destroyed);
        let next_angle = keys_iter.next().expect("Repeating");
        debug!("Aligning at angle {}", next_angle);

        debug!("Targets: {:?}", laser_queue.get(next_angle));

        if let Some(ref mut astroids_in_angle) = laser_queue.get_mut(next_angle) {
            if let Some(astroid) = astroids_in_angle.pop() {
                number_of_astroids_destroyed += 1;
                last_destroyed = Some(astroid.0.coord)
            }
        }
    }

    let last_result = last_destroyed.unwrap();
    Ok((last_result, (last_result.0 * 100 + last_result.1) as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input(
                "\
    .#..#
.....
#####
....#
...##"
            ),
            vec![
                (1, 0),
                (4, 0),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (4, 2),
                (4, 3),
                (3, 4),
                (4, 4)
            ]
        )
    }

    #[test]
    fn test_angle() {
        assert_eq!(angle((11, 13), (11, 17)), angle((11, 13), (11, 14)));
    }

    #[test]
    fn test_angle_2() {
        assert_eq!(angle((11, 13), (11, 4)), 0.0);
    }

    #[test]
    fn test_angle_3() {
        assert!(angle((11, 13), (12, 1)) < angle((11, 13), (10, 1)));
    }

    #[test]
    fn test_angle_4() {
        assert_eq!(angle((11, 13), (12, 1)), 0.0);
    }

    #[test]
    fn test_part1_210() {
        env_logger::try_init().ok();

        assert_eq!(
            part_1(
                "
.#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
"
            )
            .unwrap()
            .1,
            210
        )
    }

    #[test]
    fn test_part2() {
        env_logger::try_init().ok();

        assert_eq!(
            part_2(
                "
.#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
"
            )
            .unwrap(),
            (8, 2)
        )
    }

    #[test]
    fn test_part1_41() {
        env_logger::try_init().ok();

        assert_eq!(
            part_1(
                "
.#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..
"
            )
            .unwrap()
            .1,
            41
        );
    }
    #[test]
    fn test_part1() {
        assert_eq!(
            part_1(
                "
.#..#
.....
#####
....#
...##
"
            )
            .unwrap()
            .1,
            8
        );
        assert_eq!(
            part_1(
                "
......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####
"
            )
            .unwrap()
            .1,
            33
        );
    }
}
