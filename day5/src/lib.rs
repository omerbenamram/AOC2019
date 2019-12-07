use anyhow::Result;
use intcode_computer::IntcodeComputer;

pub fn part_1(input: &str) -> Result<String> {
    let bytecode = IntcodeComputer::parse_program(input)?;
    let mut computer = IntcodeComputer::new(bytecode);
    computer.write_to_input(vec![1])?;
    computer.run_until_halt()?;

    Ok(format!("{:?}", computer.into_output()))
}

pub fn part_2(input: &str) -> Result<String> {
    let bytecode = IntcodeComputer::parse_program(input)?;
    let mut computer = IntcodeComputer::new(bytecode);
    computer.write_to_input(vec![5])?;
    computer.run_until_halt()?;

    Ok(format!("{:?}", computer.into_output()))
}
