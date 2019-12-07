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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($i:expr, $expected:expr) => {
            let mut bytecode = ByteCodeInterpreter::from_str($i).unwrap();
            bytecode.run_until_halt().unwrap();

            assert_eq!(bytecode.to_string(), $expected.to_owned());
        };
    }

    #[test]
    fn test_computer() {
        test!("1,0,0,0,99", "2,0,0,0,99");
        test!("2,3,0,3,99", "2,3,0,6,99");
        test!("2,4,4,5,99,0", "2,4,4,5,99,9801");
        test!("1,1,1,4,99,5,6,0,99", "30,1,1,4,2,5,6,0,99");
    }
}
