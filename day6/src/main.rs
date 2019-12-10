use anyhow::{Context, Result};
use day6::{part_1, part_2};
use env_logger;
use std::io::{stdin, Read};
use std::time::Instant;

fn main() -> Result<()> {
    env_logger::init();

    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .context("Failed to read input from stdin")?;

    let now = Instant::now();
    println!("Part 1 - {} in {:?}", part_1(&input)?, now.elapsed());
    let now = Instant::now();
    println!("Part 2 - {} in {:?}", part_2(&input)?, now.elapsed());

    Ok(())
}
