#[deny(unused_must_use)]
use anyhow::{Context, Result};
use intcode_computer::{ExecutionStatus, IntcodeComputer};
use itertools::Itertools;
use log::debug;

pub fn part_1(input: &str) -> Result<i64> {
    let program = IntcodeComputer::parse_program(input)?;

    (0..=4)
        .permutations(5)
        .filter_map(|perm| calculate_thruster_signal(program.clone(), perm).ok())
        .max()
        .context("Expected a maximum")
}

pub fn part_2(input: &str) -> Result<i64> {
    let program = IntcodeComputer::parse_program(input)?;

    (5..=9)
        .permutations(5)
        .filter_map(|perm| calculate_thruster_with_feedback_loop(program.clone(), perm).ok())
        .max()
        .context("Expected a maximum")
}

fn calculate_thruster_signal(program: Vec<i64>, inputs: Vec<i64>) -> Result<i64> {
    // We have 5 amplifiers.
    let mut last_input = 0;
    let mut amps: Vec<IntcodeComputer> = (0..=4)
        .map(|_| IntcodeComputer::from_program_without_extra_memory(program.clone()))
        .collect();

    for (i, amp) in amps.iter_mut().enumerate() {
        let input = vec![inputs[i], last_input];
        debug!("Amplifier {} - {:?}", i, &input);
        amp.write_to_input(input)?;
        amp.run_until_halt()?;

        last_input = amp.read_from_output()?;
    }

    Ok(last_input)
}

fn calculate_thruster_with_feedback_loop(program: Vec<i64>, inputs: Vec<i64>) -> Result<i64> {
    let mut last_input = 0;
    let mut done = false;

    let mut amps: Vec<IntcodeComputer> = (0..=4)
        .map(|_| IntcodeComputer::from_program_without_extra_memory(program.clone()))
        .collect();

    // Load settings
    debug!("Loading settings to amplifiers.");

    for (i, amp) in amps.iter_mut().enumerate() {
        debug!("Amplifier {} - Input is {:?}", i, vec![inputs[i]]);
        amp.write_to_input(vec![inputs[i]])?;
    }

    debug!("---------- START ------------------");

    while !done {
        for (amp, i) in amps.iter_mut().zip(5..=9) {
            debug!("Amplifier {} - Input is {:?}", i, last_input);
            amp.write_to_input(vec![last_input])?;

            match amp.run()? {
                ExecutionStatus::NeedInput => {
                    debug!("Amplifier {} needs input", i);
                }
                ExecutionStatus::Done => {
                    debug!("Amplifier {} is done", i);
                    done = true
                }
            }

            last_input = amp.read_from_output()?;

            debug!("Output from Amplifier {} is {:?}", i, last_input);
        }
    }

    Ok(last_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thruster_signal() {
        assert_eq!(
            calculate_thruster_signal(
                IntcodeComputer::parse_program("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0")
                    .unwrap(),
                vec![4, 3, 2, 1, 0]
            )
            .unwrap(),
            43210
        )
    }

    #[test]
    fn test_thruster_signal_with_feedback() {
        env_logger::init();
        assert_eq!(
            calculate_thruster_with_feedback_loop(
                IntcodeComputer::parse_program(
                    "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5"
                )
                .unwrap(),
                vec![9, 8, 7, 6, 5]
            )
            .unwrap(),
            139629729
        )
    }
}
