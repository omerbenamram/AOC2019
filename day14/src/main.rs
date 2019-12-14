use anyhow::{Context, Result};
use day14::{part_1, part_2};
use std::io::{stdin, Read};
use std::time::Instant;

fn main() -> Result<()> {
    env_logger::init();

    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .context("Failed to read input from stdin")?;

    println!("Part 1 - {}", part_1(&input)?);
    let now = Instant::now();
    println!("Part 2 - {} in {:?}", part_2(&input)?, now.elapsed());

    Ok(())
}
