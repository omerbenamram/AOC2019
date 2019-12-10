use anyhow::{bail, Context, Result};
use aoc_graph::Graph;
use itertools::Itertools;
use log::debug;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

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

pub fn part_1(input: &str) -> Result<usize> {
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

            // We need to take care of special case of `0` (where slope is 0 but both left and right need to be considered)
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

    Ok(visibility_graph
        .iter()
        .map(|(v, edges)| edges.len())
        .max()
        .context("Inconclusive maximum")?)
}

pub fn part_2(input: &str) -> Result<u32> {
    Ok((0))
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
            .unwrap(),
            210
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
            .unwrap(),
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
            .unwrap(),
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
            .unwrap(),
            33
        );
    }
}
