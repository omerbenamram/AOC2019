use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use log::debug;
use std::convert::{TryFrom, TryInto};
use std::fmt;

mod io_wrapper;

pub use io_wrapper::Io;
use std::collections::VecDeque;

pub type Memory = Vec<i32>;
pub type Address = i32;

#[derive(Debug)]
enum BinaryOperation {
    Addition,
    Multiplication,
    /// if the first parameter is equal to the second parameter,
    /// it stores 1 in the position given by the third parameter.
    /// Otherwise, it stores 0
    Equals,
    /// if the first parameter is less than the second parameter,
    /// it stores 1 in the position given by the third parameter.
    /// Otherwise, it stores 0.
    LessThan,
}

impl TryFrom<u8> for BinaryOperation {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BinaryOperation::Addition),
            2 => Ok(BinaryOperation::Multiplication),
            7 => Ok(BinaryOperation::LessThan),
            8 => Ok(BinaryOperation::Equals),
            _ => bail!("Unknown arithmetic operation `{}`", value),
        }
    }
}

#[derive(Debug)]
enum UnaryOperation {
    /// Takes a single integer as input and saves it to the position given by its only parameter.
    /// For example, the instruction 3,50 would take an input value and store it at address 50
    Store,
    /// Outputs the value of its only parameter.
    /// For example, the instruction 4,50 would output the value at address 50
    Output,
}

impl TryFrom<u8> for UnaryOperation {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(UnaryOperation::Store),
            4 => Ok(UnaryOperation::Output),
            _ => bail!("Unknown unary opeartion `{}`", value),
        }
    }
}

#[derive(Debug)]
enum JumpOperation {
    JumpIfTrue,
    JumpIfFalse,
}

impl TryFrom<u8> for JumpOperation {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            5 => Ok(JumpOperation::JumpIfTrue),
            6 => Ok(JumpOperation::JumpIfFalse),
            _ => bail!("Unknown jump opeartion `{}`", value),
        }
    }
}

#[derive(Debug)]
enum ParameterMode {
    /// Causes the parameter to be interpreted as a position.
    Position,
    /// Causes the parameter to be interpreted as a value.
    Immediate,
}

impl Default for ParameterMode {
    fn default() -> Self {
        ParameterMode::Position
    }
}

impl TryFrom<u8> for ParameterMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ParameterMode::Position),
            1 => Ok(ParameterMode::Immediate),
            _ => bail!("Unknown parameter mode `{}`", value),
        }
    }
}

fn iter_digits_reversed(mut n: i32) -> impl Iterator<Item = u8> {
    debug!("iter_digits_reversed: {}", n);
    let mut n_digits_returned = 0;
    std::iter::from_fn(move || match n {
        0 => {
            if n_digits_returned >= 4 {
                None
            } else {
                n_digits_returned += 1;
                Some(0)
            }
        }
        _ => Some({
            let digit = n % 10;
            n_digits_returned += 1;
            n /= 10;
            digit as _
        }),
    })
}

#[derive(Debug)]
enum OpCode {
    Binary {
        left: ParameterMode,
        right: ParameterMode,
        t: BinaryOperation,
    },
    Unary {
        value: ParameterMode,
        t: UnaryOperation,
    },
    Jump {
        left: ParameterMode,
        right: ParameterMode,
        t: JumpOperation,
    },
    Halt,
}

impl OpCode {
    pub fn length(&self) -> i32 {
        match self {
            OpCode::Binary { .. } => 4,
            OpCode::Unary { .. } => 2,
            OpCode::Jump { .. } => 3,
            OpCode::Halt => 1,
        }
    }
}

impl TryFrom<i32> for OpCode {
    type Error = Error;

    fn try_from(n: i32) -> Result<Self> {
        let opcode: Vec<u8> = iter_digits_reversed(n).collect();
        debug!("Opcode {:?}", opcode);
        // Safety: iter_digits pads input to ensure safe access in this function.
        let op = opcode[0] + opcode[1] * 10;

        match op {
            op if BinaryOperation::try_from(op).is_ok() => {
                let left_parameter_mode = ParameterMode::try_from(opcode[2]).unwrap_or_default();

                let right_parameter_mode = ParameterMode::try_from(opcode[3]).unwrap_or_default();

                Ok(OpCode::Binary {
                    left: left_parameter_mode,
                    right: right_parameter_mode,
                    t: op.try_into()?,
                })
            }
            op if UnaryOperation::try_from(op).is_ok() => {
                let parameter_mode = ParameterMode::try_from(opcode[2]).unwrap_or_default();

                Ok(OpCode::Unary {
                    value: parameter_mode,
                    t: op.try_into()?,
                })
            }
            op if JumpOperation::try_from(op).is_ok() => {
                let left_parameter_mode = ParameterMode::try_from(opcode[2]).unwrap_or_default();
                let right_parameter_mode = ParameterMode::try_from(opcode[3]).unwrap_or_default();

                Ok(OpCode::Jump {
                    left: left_parameter_mode,
                    right: right_parameter_mode,
                    t: op.try_into()?,
                })
            }
            99 => Ok(OpCode::Halt),
            _ => bail!("`{:?}` is not a valid opcode.", opcode),
        }
    }
}

#[derive(Debug)]
pub struct IntcodeComputer {
    memory: Memory,
    io: Io,
    eip: i32,
}

#[derive(Debug)]
pub enum ExecutionStatus {
    NeedInput,
    Done,
}

impl IntcodeComputer {
    pub fn new(program: Memory) -> Self {
        Self {
            memory: program,
            io: Io::new(),
            eip: 0,
        }
    }

    pub fn parse_program(input: &str) -> Result<Memory> {
        input
            .trim_end()
            .split(",")
            .map(|number| {
                number
                    .parse::<i32>()
                    .with_context(|| format!("Expected a number `{}`", number))
            })
            .collect::<Result<Vec<i32>>>()
    }

    pub fn get(&self, i: Address) -> Result<i32> {
        self.memory
            .get(i as usize)
            .with_context(|| {
                format!(
                    "Out of bounds access while reading from memory at index `{}`",
                    i
                )
            })
            .map(|i| *i)
    }

    pub fn set_addr(&mut self, i: Address, value: i32) -> Result<()> {
        let stored = self.memory.get_mut(i as usize).with_context(|| {
            format!(
                "Out of bounds access while writing to memory at index `{}`",
                i
            )
        })?;

        *stored = value;
        Ok(())
    }

    pub fn write_to_input(&mut self, value: Vec<i32>) -> Result<()> {
        for i in value {
            self.io.input_write(i)?;
        }

        Ok(())
    }

    pub fn read_from_output(&mut self) -> Result<i32> {
        self.io.output_read()
    }

    pub fn into_output(self) -> VecDeque<i32> {
        self.io.into_output()
    }

    pub fn run(&mut self) -> Result<ExecutionStatus> {
        loop {
            // OpCode is always first two digits of number at `i`.
            let op = OpCode::try_from(self.get(self.eip)?)?;
            debug!("{:?}", &op);

            match &op {
                OpCode::Binary { left, right, t } => {
                    let (n1, n2, dest) = (
                        self.get(self.eip + 1)?,
                        self.get(self.eip + 2)?,
                        self.get(self.eip + 3)?,
                    );

                    let param1 = match left {
                        ParameterMode::Position => self.get(n1)?,
                        ParameterMode::Immediate => n1,
                    };

                    let param2 = match right {
                        ParameterMode::Position => self.get(n2)?,
                        ParameterMode::Immediate => n2,
                    };

                    match t {
                        BinaryOperation::Addition => {
                            let result = param1 + param2;
                            self.set_addr(dest, result)?;
                        }
                        BinaryOperation::Multiplication => {
                            let result = param1 * param2;
                            self.set_addr(dest, result)?;
                        }
                        BinaryOperation::Equals => {
                            if param1 == param2 {
                                self.set_addr(dest, 1)?;
                            } else {
                                self.set_addr(dest, 0)?;
                            }
                        }
                        BinaryOperation::LessThan => {
                            if param1 < param2 {
                                self.set_addr(dest, 1)?;
                            } else {
                                self.set_addr(dest, 0)?;
                            }
                        }
                    };
                    self.eip += op.length()
                }
                OpCode::Unary { value: _, t } => {
                    let dest = self.get(self.eip + 1)?;

                    match t {
                        UnaryOperation::Output => {
                            self.io.write(self.get(dest)?)?;
                        }
                        UnaryOperation::Store => match self.io.read() {
                            Err(_) => return Ok(ExecutionStatus::NeedInput),
                            Ok(i) => {
                                debug!("Setting index {} to {}", dest, i);
                                self.set_addr(dest, i)?;
                            }
                        },
                    };
                    self.eip += op.length()
                }
                OpCode::Jump { left, right, t } => {
                    let (n1, n2) = (self.get(self.eip + 1)?, self.get(self.eip + 2)?);

                    let param1 = match left {
                        ParameterMode::Position => self.get(n1)?,
                        ParameterMode::Immediate => n1,
                    };

                    let param2 = match right {
                        ParameterMode::Position => self.get(n2)?,
                        ParameterMode::Immediate => n2,
                    };

                    match t {
                        JumpOperation::JumpIfTrue => {
                            if param1 != 0 {
                                self.eip = param2
                            } else {
                                self.eip += op.length()
                            }
                        }
                        JumpOperation::JumpIfFalse => {
                            if param1 == 0 {
                                self.eip = param2
                            } else {
                                self.eip += op.length()
                            }
                        }
                    }

                    if self.eip >= self.memory.len() as i32 {
                        bail!("Segfault. EIP is at {}", self.eip);
                    }
                }
                OpCode::Halt => break,
            }
        }

        Ok(ExecutionStatus::Done)
    }

    pub fn run_until_halt(&mut self) -> Result<()> {
        match self.run() {
            Ok(status) => match status {
                ExecutionStatus::NeedInput => return Err(Error::msg("EOF")),
                ExecutionStatus::Done => Ok(()),
            },
            Err(e) => Err(e),
        }
    }
}

impl fmt::Display for IntcodeComputer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = self.memory.iter().join(",");
        f.write_str(&repr)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_memory {
        ($prog: expr, $i:expr, $o: expr) => {
            let bytecode = IntcodeComputer::parse_program($prog).unwrap();
            let q = VecDeque::from($i);
            let mut computer = IntcodeComputer::new(bytecode);

            computer.run_until_halt().unwrap();
            assert_eq!(computer.to_string(), $o);
        };
    }

    macro_rules! test_output {
        ($prog: expr, $i:expr, $o: expr) => {
            let bytecode = IntcodeComputer::parse_program($prog).unwrap();
            let q = VecDeque::from($i);
            let mut computer = IntcodeComputer::new(bytecode);

            computer.run_until_halt().unwrap();

            assert_eq!(computer.into_output(), $o);
        };
    }

    #[test]
    fn test_computer_new() {
        test_memory!("1002,4,3,4,33", vec![1], "1002,4,3,4,99");
    }

    #[test]
    fn test_computer_input() {
        test_output!("3,9,8,9,10,9,4,9,99,-1,8", vec![0], vec![0]);
        test_output!("3,9,8,9,10,9,4,9,99,-1,8", vec![8], vec![1]);
        test_output!("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", vec![0], vec![0]);
    }

    #[test]
    fn test_computer_input_immediate() {
        test_output!("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", vec![0], vec![0]);
    }
}
