use anyhow::{bail, Context, Error, Result};
use intcode_computer::{ExecutionStatus, IntcodeComputer, Io};
use itertools::Itertools;
use log::debug;
use std::collections::VecDeque;

pub fn part_1(input: &str) -> Result<i32> {
    let program = IntcodeComputer::parse_program(input)?;

    Ok((0..=4)
        .permutations(5)
        .filter_map(|perm| calculate_thruster_signal(program.clone(), perm).ok())
        .max()
        .expect("There is a maximum"))
}

pub fn part_2(input: &str) -> Result<i32> {
    let program = IntcodeComputer::parse_program(input)?;

    Ok((5..=9)
        .permutations(5)
        .filter_map(|perm| calculate_thruster_signal(program.clone(), perm).ok())
        .max()
        .expect("There is a maximum"))
}

#[derive(Debug)]
struct Amplifier {
    pub amplifier_id: i32,
    program: IntcodeComputer,
}

impl Amplifier {
    pub fn new(id: i32, program: Vec<i32>) -> Self {
        let computer = IntcodeComputer::new(program);

        Self {
            amplifier_id: id,
            program: computer,
        }
    }
    pub fn provide_input(&mut self, input: Vec<i32>) -> Result<()> {
        self.program.write_to_input(input);
        Ok(())
    }

    pub fn run(&mut self) -> ExecutionStatus {
        self.program.run()
    }

    pub fn read_from_output(&mut self) -> Result<i32> {
        self.program.read_from_output()
    }
}

fn calculate_thruster_signal(program: Vec<i32>, inputs: Vec<i32>) -> Result<i32> {
    // We have 5 amplifiers.
    let mut last_input = 0;
    let mut amps: Vec<Amplifier> = (0..=4)
        .map(|id| Amplifier::new(id, program.clone()))
        .collect();

    for amp in amps.iter_mut() {
        let input = vec![inputs[amp.amplifier_id as usize], last_input];
        debug!("Amplifier {} - {:?}", amp.amplifier_id, &input);
        amp.provide_input(input);
        match amp.run() {
            ExecutionStatus::Error(e) => return Err(e),
            ExecutionStatus::NeedInput => continue,
            ExecutionStatus::Done => {}
        }
        last_input = amp.read_from_output()?;
    }

    Ok(last_input)
}

fn calculate_thruster_with_feedback_loop(program: Vec<i32>, inputs: Vec<i32>) -> Result<i32> {
    let mut last_input = 0;
    let mut done = false;

    let mut amps: Vec<Amplifier> = (5..=9)
        .map(|id| Amplifier::new(id, program.clone()))
        .collect();

    // Bootstrap feedback loop process
    let amp_0 = amps.get_mut(0).expect("Amp exists");
    amp_0.provide_input(vec![0]);
    amp_0.run();

    // Load settings
    for amp in amps.iter_mut() {
        amp.provide_input(vec![inputs[(amp.amplifier_id - 5) as usize]]);
    }

    while !done {
        for amp in amps.iter_mut() {
            let input = vec![last_input];
            debug!("Amplifier {} - Input is {:?}", amp.amplifier_id, &input);
            amp.provide_input(input);

            match amp.run() {
                ExecutionStatus::Error(e) => return Err(e),
                ExecutionStatus::NeedInput => {
                    debug!("Amplifier {} needs input", amp.amplifier_id);
                }
                ExecutionStatus::Done => {
                    debug!("Amplifier {} is done", amp.amplifier_id);
                    done = true
                }
            }
            last_input = amp.read_from_output()?;
            debug!(
                "Output from Amplifier {} is {}",
                amp.amplifier_id, last_input
            );
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
                IntcodeComputer::parse_program("3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5")
                    .unwrap(),
                vec![9,8,7,6,5]
            )
            .unwrap(),
            139629729
        )
    }
}
