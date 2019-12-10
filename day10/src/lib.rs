use anyhow::{bail, Context, Result};
use itertools::Itertools;
use log::debug;
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

fn distance(a: Coord, b: Coord) -> f32 {
    (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f32).sqrt()
}

fn angle(a: &Coord, b: &Coord) -> f32 {
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

/// How many different angles can we see from our asteroid?
fn count_visible_astroids(astroids: &Vec<Coord>, asteroid: &Coord) -> usize {
    let mut visible_count = HashSet::new();

    for another in astroids.iter() {
        if asteroid == another {
            continue;
        }

        let angle = (angle(asteroid, another) * 10000.0) as i32;
        visible_count.insert(angle);
    }

    visible_count.len()
}

pub fn part_1(input: &str) -> Result<(Coord, usize)> {
    let astroids = parse_input(input);

    if astroids.is_empty() {
        bail!("Input is empty.");
    }

    let max: (Coord, usize) = astroids
        .iter()
        .map(|&astroid| (astroid, count_visible_astroids(&astroids, &astroid)))
        .max_by_key(|(_v, visible_count)| visible_count.clone())
        .context("Inconclusive maximum")?;

    Ok((max.0, max.1 as usize))
}

#[derive(Eq, Debug, Clone)]
struct AsteroidWithDistance {
    coord: Coord,
    distance: i32,
}

impl PartialEq for AsteroidWithDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl PartialOrd for AsteroidWithDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Ord for AsteroidWithDistance {
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

    let (start, _) = part_1(input)?;

    // {Angle -> [Vertex Sorted By Distance]}
    let mut laser_queue = HashMap::new();
    // Cannot use angle as f32 key, but we still need to know the order..
    let mut all_angles = Vec::new();

    // Build slope graph
    for another in asteroids.iter().cloned() {
        if another == start {
            continue;
        }
        let angle = angle_abs(start, another);
        let distance = distance(start, another) * 10000.0;
        let ast = AsteroidWithDistance {
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
    let total_astroids = asteroids.len();
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
