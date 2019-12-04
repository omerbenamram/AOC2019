use anyhow::{Context, Result};
use day4::{parse_range, part_1, part_2};
use std::io::{stdin, Read};

fn main() -> Result<()> {
    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .context("Failed to read input from stdin")?;

    let (low, hi) = parse_range(&input)?;

    println!("Part 1 - {}", part_1(low, hi));
    println!("Part 2 - {}", part_2(low, hi));

    Ok(())
}
