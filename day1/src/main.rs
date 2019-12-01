use anyhow::{Context, Result};
use day1::{part_1, part_2};
use std::io::{stdin, Read};

fn main() -> Result<()> {
    let mut input = vec![];

    stdin()
        .read_to_end(&mut input)
        .context("Failed to read input from stdin")?;

    let input_as_string =
        String::from_utf8(input).context("String contains Non-UTF8 characters")?;

    println!("Part 1 - {}", part_1(&input_as_string)?);
    println!("Part 2 - {}", part_2(&input_as_string)?);

    Ok(())
}
