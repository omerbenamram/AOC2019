use anyhow::{Context, Result};
use day11::{part_1, part_2};
use std::io::{stdin, Read};

fn main() -> Result<()> {
    env_logger::init();

    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .context("Failed to read input from stdin")?;

    println!("Part 1 - {}", part_1(&input)?);
    println!("Part 2 - {}", part_2(&input)?);

    Ok(())
}
