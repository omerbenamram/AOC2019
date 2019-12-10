use anyhow::{bail, Context, Result};
use itertools::Itertools;
use log::debug;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

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

fn distance(a: &Coord, b: &Coord) -> f32 {
    (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f32).sqrt()
}

fn angle<T: Borrow<Coord>>(a: T, b: T) -> f32 {
    let (a, b) = (a.borrow(), b.borrow());
    let dx = (a.0 - b.0) as f32;
    let dy = (a.1 - b.1) as f32;
    dx.atan2(dy)
}

/// Absolute angle relative to "up" direction of canon.
fn angle_abs<T: Borrow<Coord>>(a: T, b: T) -> f32 {
    let (a, b) = (a.borrow(), b.borrow());
    let dx = (a.0 - b.0) as f32;
    let dy = (a.1 - b.1) as f32;
    let atan = dx.atan2(dy);

    if atan > 0.0 {
        (2.0 * std::f32::consts::PI - atan)
    } else {
        atan * -1.0
    }
}

/// How many different angles can we see from our asteroid?
fn count_visible_asteroids(asteroids: &Vec<Coord>, asteroid: &Coord) -> usize {
    let mut visible_count = HashSet::new();

    for another in asteroids.iter() {
        if asteroid == another {
            continue;
        }

        let angle = (angle(asteroid, another) * 10000.0) as i32;
        visible_count.insert(angle);
    }

    visible_count.len()
}

fn best_asteroid(asteroids: &Vec<Coord>) -> Result<(Coord, usize)> {
    asteroids
        .iter()
        .map(|&astroid| (astroid, count_visible_asteroids(&asteroids, &astroid)))
        .max_by_key(|(_v, visible_count)| visible_count.clone())
        .context("Inconclusive maximum")
}

pub fn part_1(input: &str) -> Result<(Coord, usize)> {
    let asteroids = parse_input(input);

    if asteroids.is_empty() {
        bail!("Input is empty.");
    }

    best_asteroid(&asteroids)
}

#[derive(Eq, Debug, Clone)]
struct Target {
    coord: Coord,
    distance: i32,
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl PartialOrd for Target {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Ord for Target {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(Ordering::Equal)
    }
}

pub fn part_2(input: &str) -> Result<(Coord, usize)> {
    let asteroids = parse_input(input);

    if asteroids.is_empty() {
        bail!("Input is empty.");
    }

    let (origin, _) = best_asteroid(&asteroids)?;

    // {Angle -> (Minimum)BinaryHeap[Vertex sorted by distance]}.
    let mut laser_queue = HashMap::new();

    // Clone the len here since we move out of `asteroids`.
    let total_asteroids = asteroids.len();

    for asteroid in asteroids.into_iter() {
        if asteroid == origin {
            continue;
        }

        let angle_key = (angle_abs(&origin, &asteroid) * 10000.0).round() as i32;
        let distance_key = (distance(&origin, &asteroid) * 10000.0).round() as i32;

        let target = Target {
            coord: asteroid,
            distance: distance_key,
        };

        laser_queue
            .entry(angle_key)
            .or_insert_with(BinaryHeap::new)
            .push(std::cmp::Reverse(target));
    }

    let mut keys_iter = laser_queue.keys().cloned().sorted().cycle();
    let mut number_of_astroids_destroyed = 0;
    let mut last_destroyed = None;

    while (number_of_astroids_destroyed < 200) && (number_of_astroids_destroyed <= total_asteroids)
    {
        debug!("{} -> {:?}", number_of_astroids_destroyed, last_destroyed);
        let next_angle = keys_iter.next().expect("This iterator is repeating");

        debug!("Aligning at angle {}", next_angle);
        debug!("Targets: {:?}", laser_queue.get(&next_angle));

        if let Some(ref mut astroids_in_angle) = laser_queue.get_mut(&next_angle) {
            if let Some(std::cmp::Reverse(asteroid)) = astroids_in_angle.pop() {
                number_of_astroids_destroyed += 1;
                last_destroyed = Some(asteroid.coord)
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
            ((8, 2), 802)
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
