#![deny(unused_must_use)]

use anyhow::{bail, Context, Result};
use intcode_computer::{ExecutionStatus, IntcodeComputer};
use log::debug;
use petgraph::prelude::*;
use regex::Regex;
use std::cmp;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::thread;
use std::time::Duration;

const ONE_TRILLION: usize = 1_000_000_000_000;

type Chemical = (String, usize);
type Reaction = (Chemical, Vec<Chemical>);
type ReactionsMap = HashMap<String, Reaction>;

fn parse_input(input: &str) -> Result<ReactionsMap> {
    let mut reactions = vec![];
    let reg = Regex::new(r#"(\d+) (\w+),?"#).unwrap();

    for line in input.trim().lines() {
        // 7 A, 1 B => 1 C
        let parse: Vec<&str> = line.split("=").collect();
        let rhs = reg.captures(parse[1]).context("invalid input")?;

        let chemical = rhs[2].to_string();
        let cost = rhs[1].parse::<usize>()?;

        let mut dependencies: Vec<Chemical> = vec![];

        for dep in reg.captures_iter(parse[0]) {
            let chemical = dep[2].to_string();
            let cost = dep[1].parse::<usize>()?;
            dependencies.push((chemical, cost));
        }

        reactions.push(((chemical, cost), dependencies));
    }

    let reactions_map = reactions
        .into_iter()
        .map(|(chem, deps)| (chem.0.clone(), (chem, deps)))
        .collect::<HashMap<String, Reaction>>();

    Ok(reactions_map)
}

pub fn part_1(input: &str) -> Result<usize> {
    let reactions = parse_input(input)?;
    Ok(computer_ore_needed_for_fuel(1, &reactions))
}

/// This is basically a modified DFS with some state.
fn computer_ore_needed_for_fuel(how_much: usize, reactions: &ReactionsMap) -> usize {
    let mut have = HashMap::new();
    let mut needed = Vec::new();
    let mut total_ore = 0;

    needed.push(("FUEL", how_much));

    while let Some((product, mut quantity)) = needed.pop() {
        if let Some(a) = have.get(product).cloned() {
            let consumed = quantity.min(a);
            have.insert(product, a - consumed);
            quantity -= consumed;
        }
        if quantity > 0 {
            let recipe = &reactions[product];
            let amount = (quantity + (recipe.0).1 - 1) / (recipe.0).1;
            for (p, q) in &recipe.1 {
                if p == "ORE" {
                    total_ore += q * amount;
                } else {
                    needed.push((p, q * amount));
                }
            }
            have.insert(product, (recipe.0).1 * amount - quantity);
        }
    }

    total_ore
}

/// bisect to find correct amount..
pub fn part_2(input: &str) -> Result<usize> {
    let reactions = parse_input(input)?;
    // maximum possibly needed is ONE_TRILLION divided by amount of ore for one fuel.
    // there might be a better solution though.
    let ore_for_1_fuel = ONE_TRILLION / computer_ore_needed_for_fuel(1, &reactions);

    let (mut min, mut max) = (ore_for_1_fuel, ore_for_1_fuel * 2);

    while min != max {
        let middle = (min + max + 1) / 2;
        match computer_ore_needed_for_fuel(middle, &reactions).cmp(&ONE_TRILLION) {
            Ordering::Equal => return Ok(middle),
            Ordering::Greater => max = (min + max) / 2,
            Ordering::Less => min = middle,
        }
    }

    Ok(max)
}

#[cfg(test)]
mod tests {
    use crate::part_1;

    #[test]
    fn test_part1() {
        env_logger::try_init().ok();
        assert_eq!(
            part_1(
                "10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL"
            )
            .unwrap(),
            31
        );
    }

    #[test]
    fn test_another_part1() {
        assert_eq!(
            part_1(
                "9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL"
            )
            .unwrap(),
            165
        )
    }

    #[test]
    fn test_another_2_part1() {
        env_logger::try_init().ok();
        assert_eq!(
            part_1(
                "157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"
            )
            .unwrap(),
            13312
        )
    }
}
