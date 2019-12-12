#![deny(unused_must_use)]

use anyhow::{bail, Context, Error, Result};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;
use std::{fmt, ops};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct Point3 {
    x: i32,
    y: i32,
    z: i32,
}

impl fmt::Display for Point3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "<x={x:3}, y={y:3}, z={z:3}>",
            x = self.x,
            y = self.y,
            z = self.z
        ))
    }
}

impl ops::AddAssign<&Point3> for Point3 {
    fn add_assign(&mut self, rhs: &Point3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
struct Moon {
    position: Point3,
    velocity: Point3,
}

impl Moon {
    pub fn apply_gravity(&mut self, other: &Moon) {
        match self.position.x.cmp(&other.position.x) {
            Ordering::Less => self.velocity.x += 1,
            Ordering::Greater => self.velocity.x -= 1,
            Ordering::Equal => {}
        }

        match self.position.y.cmp(&other.position.y) {
            Ordering::Less => self.velocity.y += 1,
            Ordering::Greater => self.velocity.y -= 1,
            Ordering::Equal => {}
        }
        match self.position.z.cmp(&other.position.z) {
            Ordering::Less => self.velocity.z += 1,
            Ordering::Greater => self.velocity.z -= 1,
            Ordering::Equal => {}
        }
    }

    pub fn apply_velocity(&mut self) {
        self.position += &self.velocity
    }

    pub fn potential_energy(&self) -> i32 {
        let potential_energy =
            self.position.x.abs() + self.position.y.abs() + self.position.z.abs();
        let kinetic_energy = self.velocity.x.abs() + self.velocity.y.abs() + self.velocity.z.abs();
        potential_energy * kinetic_energy
    }
}

impl fmt::Display for Moon {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("pos={}, vel={}", self.position, self.velocity))
    }
}

impl FromStr for Moon {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref MOON_RE: Regex =
                Regex::new(r#"\w=(?P<position>-?\d+),?\s?"#).expect("A valid regex");
        }

        let coords: Vec<i32> = MOON_RE
            .captures_iter(s)
            .filter_map(|c| c.name("position"))
            .map(|p| p.as_str().parse::<i32>().expect("Matched a number"))
            .collect();

        if coords.len() != 3 {
            bail!("Invalid input `{}`, expected exactly three numbers", s);
        }

        Ok(Moon {
            position: Point3 {
                x: coords[0],
                y: coords[1],
                z: coords[2],
            },
            velocity: Point3::default(),
        })
    }
}

pub fn part_1(input: &str) -> Result<i32> {
    let mut moons: Vec<Moon> = input
        .lines()
        .map(Moon::from_str)
        .collect::<Result<Vec<Moon>>>()?;

    if moons.is_empty() {
        bail!("Expected input");
    }

    for generation in 0..1000 {
        if log::log_enabled!(log::Level::Debug) {
            for moon in moons.iter() {
                println!("{}", moon);
            }
            println!("---------------------")
        }

        let copy = moons.clone();

        for moon in moons.iter_mut() {
            for other in &copy {
                moon.apply_gravity(other);
            }
        }

        for moon in moons.iter_mut() {
            moon.apply_velocity();
        }
    }

    Ok(moons.iter().map(|moon| moon.potential_energy()).sum())
}

pub fn part_2(input: &str) -> Result<i32> {
    Ok(0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_input() {
        assert_eq!(
            Moon::from_str("<x=-3, y=-8, z=-8>").unwrap(),
            Moon {
                position: Point3 {
                    x: -3,
                    y: -8,
                    z: -8,
                },
                velocity: Default::default()
            }
        );
    }

    #[test]
    fn test_part_1() {
        assert_eq!(
            part_1(
                "<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>
"
            )
            .unwrap(),
            179
        )
    }
}
