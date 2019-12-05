use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use log::debug;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Write;
use std::io::{Cursor, Read};
use std::str::FromStr;

type Memory = Vec<i32>;

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

#[derive(Clone)]
struct IntcodeComputer(Memory);

impl IntcodeComputer {
    pub fn get(&self, i: i32) -> Result<i32> {
        self.0
            .get(i as usize)
            .with_context(|| {
                format!(
                    "Out of bounds access while reading from memory at index `{}`",
                    i
                )
            })
            .map(|i| *i)
    }

    pub fn set(&mut self, i: i32, value: i32) -> Result<()> {
        let stored = self.0.get_mut(i as usize).with_context(|| {
            format!(
                "Out of bounds access while writing to memory at index `{}`",
                i
            )
        })?;

        *stored = value;
        Ok(())
    }

    pub fn run_until_halt(&mut self, input: &Vec<i32>) -> Result<String> {
        let mut i = 0;
        let mut input_cursor = input.iter();
        let mut output = String::new();

        loop {
            // OpCode is always first two digits of number at `i`.
            let op = OpCode::try_from(self.get(i)?)?;
            debug!("{:?}", &self.0[..10]);
            debug!("{:?}", &op);

            match &op {
                OpCode::Binary { left, right, t } => {
                    let (n1, n2, dest) = (self.get(i + 1)?, self.get(i + 2)?, self.get(i + 3)?);

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
                            self.set(dest, result)?;
                        }
                        BinaryOperation::Multiplication => {
                            let result = param1 * param2;
                            self.set(dest, result)?;
                        }
                        BinaryOperation::Equals => {
                            if param1 == param2 {
                                self.set(dest, 1)?;
                            } else {
                                self.set(dest, 0)?;
                            }
                        }
                        BinaryOperation::LessThan => {
                            if param1 < param2 {
                                self.set(dest, 1)?;
                            } else {
                                self.set(dest, 0)?;
                            }
                        }
                    };
                    i += op.length()
                }
                OpCode::Unary { value: _, t } => {
                    let dest = self.get(i + 1)?;

                    match t {
                        UnaryOperation::Output => {
                            writeln!(output, "OUTPUT: {}", self.get(dest)?)?;
                        }
                        UnaryOperation::Store => {
                            let next_input = input_cursor
                                .next()
                                .with_context(|| "Unexpected end of input!")?;

                            debug!("Setting index {} to {}", dest, next_input);
                            self.set(dest, *next_input)?;
                        }
                    };
                    i += op.length()
                }
                OpCode::Jump { left, right, t } => {
                    let (n1, n2) = (self.get(i + 1)?, self.get(i + 2)?);

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
                                i = param2
                            } else {
                                i += op.length()
                            }
                        }
                        JumpOperation::JumpIfFalse => {
                            if param1 == 0 {
                                i = param2
                            } else {
                                i += op.length()
                            }
                        }
                    }

                    if i >= self.0.len() as i32 {
                        bail!("Segfault. EIP is at {}", i);
                    }
                }
                OpCode::Halt => break,
            }
        }

        Ok(output)
    }
}

impl fmt::Display for IntcodeComputer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = self.0.iter().join(",");
        f.write_str(&repr)?;

        Ok(())
    }
}

impl FromStr for IntcodeComputer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: Vec<i32> = s
            .trim_end()
            .split(",")
            .map(|number| {
                number
                    .parse::<i32>()
                    .with_context(|| format!("Expected a number `{}`", number))
            })
            .collect::<Result<Vec<i32>>>()?;

        Ok(IntcodeComputer(inner))
    }
}

pub fn part_1(input: &str) -> Result<String> {
    let mut computer = IntcodeComputer::from_str(input)?;
    computer.run_until_halt(&vec![1])
}

pub fn part_2(input: &str) -> Result<String> {
    let mut computer = IntcodeComputer::from_str(input)?;
    computer.run_until_halt(&vec![5])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computer_new() {
        let mut bytecode = IntcodeComputer::from_str("1002,4,3,4,33").unwrap();
        bytecode.run_until_halt(&vec![1]).unwrap();

        assert_eq!(bytecode.to_string(), "1002,4,3,4,99");
    }

    #[test]
    fn test_computer_input() {
        let mut bytecode = IntcodeComputer::from_str("3,9,8,9,10,9,4,9,99,-1,8").unwrap();
        let output = bytecode.run_until_halt(&vec![0]).unwrap();
        assert!(output.contains("0"));

        let mut bytecode = IntcodeComputer::from_str("3,9,8,9,10,9,4,9,99,-1,8").unwrap();
        let output = bytecode.run_until_halt(&vec![8]).unwrap();
        assert!(output.contains("1"));

        let mut bytecode =
            IntcodeComputer::from_str("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9").unwrap();
        let output = bytecode.run_until_halt(&vec![0]).unwrap();
        assert!(output.contains("0"));
    }

    #[test]
    fn test_computer_input_immediate() {
        let mut bytecode =
            IntcodeComputer::from_str("3,3,1105,-1,9,1101,0,0,12,4,12,99,1").unwrap();
        let output = bytecode.run_until_halt(&vec![0]).unwrap();
        assert!(output.contains("0"));
    }
}
