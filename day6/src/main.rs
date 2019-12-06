use anyhow::{Context, Result};
use day6::part_1;
use env_logger;
use std::io::{stdin, Read};

fn main() -> Result<()> {
    env_logger::init();

    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .context("Failed to read input from stdin")?;

    println!("Part 1 - {}", part_1(&input)?);

    Ok(())
}
