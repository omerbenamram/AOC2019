use crate::Direction::{Down, Up};
use anyhow::{bail, Context, Error, Result};
use itertools::{all, Itertools};
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ops::{Range, RangeBounds, RangeInclusive};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Direction {
    Up(i32),
    Down(i32),
    Left(i32),
    Right(i32),
}

impl FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        use Direction::*;

        let direction = s
            .chars()
            .next()
            .context("Direction should not be an empty string")?;

        let n_steps: i32 = s
            .get(1..)
            .context("Direction should contain numbers after letter")?
            .parse()?;

        let result = match direction {
            'U' => Up(n_steps),
            'D' => Down(n_steps),
            'L' => Left(n_steps),
            'R' => Right(n_steps),
            _ => bail!("Unknown direction `{}`", direction),
        };

        Ok(result)
    }
}

#[derive(Eq, Ord, PartialOrd, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Point(i32, i32);

impl Point {
    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }

    pub fn is_origin(&self) -> bool {
        (self.x(), self.y()) == (0, 0)
    }

    pub fn shift_in(&self, direction: &Direction) -> Point {
        match direction {
            Direction::Up(n) => Point(self.0, self.1 + *n),
            Direction::Down(n) => Point(self.0, self.1 - *n),
            Direction::Left(n) => Point(self.0 - *n, self.1),
            Direction::Right(n) => Point(self.0 + *n, self.1),
        }
    }

    pub fn distance(&self, other: &Point) -> i32 {
        (self.x() - other.x()).abs() + (self.y() - other.y()).abs()
    }

    pub fn manhattan_distance(&self) -> i32 {
        self.0.abs() + self.1.abs()
    }
}

#[derive(Eq, Ord, PartialOrd, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Line(Point, Point);

impl Line {
    pub fn xs(&self) -> RangeInclusive<i32> {
        let min = cmp::min(self.0.x(), self.1.x());
        let max = cmp::max(self.0.x(), self.1.x());
        min..=max
    }

    pub fn ys(&self) -> RangeInclusive<i32> {
        let min = cmp::min(self.0.y(), self.1.y());
        let max = cmp::max(self.0.y(), self.1.y());
        min..=max
    }

    pub fn len(&self) -> i32 {
        self.0.distance(&self.1)
    }

    pub fn distance_from_point(&self, p: &Point) -> i32 {
        self.0.distance(p)
    }

    pub fn is_horizontal(&self) -> bool {
        self.0.y() == self.1.y()
    }

    pub fn is_vertical(&self) -> bool {
        self.0.x() == self.1.x()
    }

    pub fn intersects_line(&self, other: &Line) -> Option<Point> {
        if self.is_horizontal() {
            // assuming this is not the same line
            if other.is_horizontal() {
                return None;
            }

            // other line is vertical, so `.xs()` start == end
            let other_x = other.xs().start().clone();

            if self.xs().contains(&other_x) {
                // search for y's intersection
                for y in other.ys() {
                    if self.ys().contains(&y) {
                        let intersection = Point(other_x, y);
                        if !intersection.is_origin() {
                            return Some(intersection);
                        }
                    }
                }
            }
        }

        if self.is_vertical() {
            // assuming this is not the same line
            if other.is_vertical() {
                return None;
            }

            // other line is horizontal, so `.ys()` start == end
            let other_y = other.ys().start().clone();

            if self.ys().contains(&other_y) {
                // search for y's intersection
                for x in other.xs() {
                    // exclude origin
                    if self.xs().contains(&x) {
                        let intersection = Point(x, other_y);
                        if !intersection.is_origin() {
                            return Some(intersection);
                        }
                    }
                }
            }
        }
        return None;
    }

    pub fn intersects_point(&self, other: &Point) -> bool {
        self.xs().contains(&other.x()) && self.ys().contains(&other.y())
    }
}

struct Wire(Vec<Direction>);

impl FromStr for Wire {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let inner: Vec<Direction> = s
            .trim()
            .split(",")
            .map(Direction::from_str)
            .collect::<Result<Vec<Direction>>>()?;
        Ok(Wire(inner))
    }
}

impl Wire {
    pub fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        let mut position = Point(0, 0);
        let mut iter = self.0.iter();

        std::iter::from_fn(move || {
            if let Some(step) = iter.next() {
                let next_position = position.shift_in(step);
                let line = Line(position, next_position);
                position = next_position;
                return Some(line);
            }
            return None;
        })
    }
}

fn find_intersections(lines_1: &Vec<Line>, lines_2: &Vec<Line>) -> HashSet<Point> {
    let mut intersections = HashSet::new();

    for l1 in lines_1.iter() {
        for l2 in lines_2.iter() {
            if let Some(point) = l1.intersects_line(&l2) {
                intersections.insert(point);
            }
        }
    }

    intersections
}

pub fn part_1(input: &str) -> Result<i32> {
    let mut wires = input
        .lines()
        .map(Wire::from_str)
        .collect::<Result<Vec<Wire>>>()?;

    if wires.len() != 2 {
        bail!("Expected two wires exactly, found {}.", wires.len())
    }

    let wire_1 = &wires[0];
    let wire_2 = &wires[1];

    let lines_1 = wire_1.iter_lines().collect();
    let lines_2 = wire_2.iter_lines().collect();

    let intersections = find_intersections(&lines_1, &lines_2);

    if intersections.is_empty() {
        bail!("Wires do not intersect.")
    }

    intersections
        .iter()
        .map(|p| p.manhattan_distance())
        .min()
        .ok_or_else(|| Error::msg("Expected a minimum"))
}

pub fn part_2(input: &str) -> Result<i32> {
    let mut wires = input
        .lines()
        .map(Wire::from_str)
        .collect::<Result<Vec<Wire>>>()?;

    if wires.len() != 2 {
        bail!("Expected two wires exactly, found {}.", wires.len())
    }

    let wire_1: &Wire = &wires[0];
    let wire_2: &Wire = &wires[1];

    let mut lines_1 = wire_1.iter_lines().collect();
    let lines_2 = wire_2.iter_lines().collect();

    let intersections = find_intersections(&lines_1, &lines_2);

    if intersections.is_empty() {
        bail!("Wires do not intersect.")
    }

    let mut intersection_to_len = HashMap::new();

    for intersection in intersections.iter() {
        let mut sum = 0;
        for line in lines_1.iter() {
            if line.intersects_point(intersection) {
                sum += line.distance_from_point(intersection);
                break;
            } else {
                sum += line.len()
            }
        }
        for line in lines_2.iter() {
            if line.intersects_point(intersection) {
                sum += line.distance_from_point(intersection);
                break;
            } else {
                sum += line.len()
            }
        }
        intersection_to_len.insert(intersection, sum);
    }

    Ok(*intersection_to_len
        .values()
        .min()
        .expect("We've checked intersections is not empty."))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lines_horizontal_vertical() {
        let l1 = Line(Point(0, 0), Point(0, 10));
        let l2 = Line(Point(-5, 5), Point(5, 5));
        // horizontal -> vertical
        assert_eq!(l2.intersects_line(&l1), Some(Point(0, 5)));
    }

    #[test]
    fn test_lines_vertical_horizontal() {
        let l1 = Line(Point(0, 0), Point(0, 10));
        let l2 = Line(Point(-5, 5), Point(5, 5));
        // vertical -> horizontal
        assert_eq!(l1.intersects_line(&l2), Some(Point(0, 5)));
    }

    #[test]
    fn test_lines_reflexiveness() {
        let l1 = Line(Point(0, 10), Point(0, 0));
        let l2 = Line(Point(-5, 5), Point(5, 5));
        assert_eq!(l1.intersects_line(&l2), Some(Point(0, 5)));
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "R8,U5,L5,D3
                    U7,R6,D4,L4"
            )
            .unwrap(),
            6
        );
        assert_eq!(
            part_1(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72
                            U62,R66,U55,R34,D71,R55,D58,R83"
            )
            .unwrap(),
            159
        )
    }

    #[test]
    fn test_part_2() {
        assert_eq!(
            part_2(
                "R8,U5,L5,D3
                    U7,R6,D4,L4"
            )
            .unwrap(),
            30
        );

        assert_eq!(
            part_2(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72
                        U62,R66,U55,R34,D71,R55,D58,R83"
            )
            .unwrap(),
            610
        );

        assert_eq!(
            part_2(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51
                        U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            )
            .unwrap(),
            410
        );
    }
}
