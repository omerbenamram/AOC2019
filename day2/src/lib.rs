use anyhow::{Context, Error, Result};
use intcode_computer::IntcodeComputer;

pub fn part_1(input: &str) -> Result<i32> {
    let memory = IntcodeComputer::parse_program(input)?;
    let mut computer = IntcodeComputer::new(memory);
    // restore gravity assist
    computer.set_addr(1, 12)?;
    computer.set_addr(2, 2)?;

    computer.run_until_halt()?;
    // Output is provided at address 0
    computer.get(0).context("Expected output")
}

pub fn part_2(input: &str) -> Result<i32> {
    let program = IntcodeComputer::parse_program(input)?;

    for noun in 0..=99 {
        for verb in 0..=99 {
            // Clone computer here to avoid reparsing input.
            let mut computer = IntcodeComputer::new(program.clone());
            computer.set_addr(1, noun)?;
            computer.set_addr(2, verb)?;

            computer.run_until_halt()?;

            if computer.get(0).context("Expected output")? == 19690720 {
                return Ok((100 * noun) + verb);
            }
        }
    }

    Err(Error::msg("Value not found."))
}
