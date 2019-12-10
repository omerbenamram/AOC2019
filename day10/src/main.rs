use anyhow::{Context, Result};
use day10::{part_1, part_2};
use env_logger;
use std::io::{stdin, Read};
use std::time::Instant;

fn main() -> Result<()> {
    env_logger::init();

    let mut input = String::new();

    stdin()
        .read_to_string(&mut input)
        .context("Failed to read input from stdin")?;

    let start = Instant::now();
    println!("Part 1 - {:?} in {:?}", part_1(&input)?, start.elapsed());
    let start = Instant::now();
    println!("Part 2 - {:?} in {:?}", part_2(&input)?, start.elapsed());

    Ok(())
}
