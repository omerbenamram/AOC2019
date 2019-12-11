#![deny(unused_must_use)]
use anyhow::{Context, Result};
use intcode_computer::IntcodeComputer;

pub fn part_1(input: &str) -> Result<i64> {
    let program = IntcodeComputer::parse_program(input)?;
    let mut computer = IntcodeComputer::new(program);

    computer.write_to_input(vec![1])?;
    computer.run_until_halt()?;

    computer
        .into_output()
        .pop_front()
        .context("Expected output")
}

pub fn part_2(input: &str) -> Result<i64> {
    let program = IntcodeComputer::parse_program(input)?;
    let mut computer = IntcodeComputer::new(program);

    computer.write_to_input(vec![2])?;
    computer.run_until_halt()?;

    computer
        .into_output()
        .pop_front()
        .context("Expected output")
}
