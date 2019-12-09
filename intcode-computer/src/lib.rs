use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use log::debug;
use std::convert::{TryFrom, TryInto};
use std::fmt;

mod io_wrapper;

pub use io_wrapper::Io;
use std::collections::VecDeque;

pub type Memory = Vec<i64>;
pub type Address = i64;

#[derive(Debug, PartialOrd, PartialEq)]
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

#[derive(Debug, PartialOrd, PartialEq)]
enum UnaryOperation {
    /// Takes a single integer as input and saves it to the position given by its only parameter.
    /// For example, the instruction 3,50 would take an input value and store it at address 50
    Store,
    /// Outputs the value of its only parameter.
    /// For example, the instruction 4,50 would output the value at address 50
    Output,
    AdjustRelativeBase,
}

impl TryFrom<u8> for UnaryOperation {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(UnaryOperation::Store),
            4 => Ok(UnaryOperation::Output),
            9 => Ok(UnaryOperation::AdjustRelativeBase),
            _ => bail!("Unknown unary opeartion `{}`", value),
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
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

#[derive(Debug, PartialOrd, PartialEq)]
enum ParameterMode {
    /// Causes the parameter to be interpreted as a position.
    Position,
    /// Causes the parameter to be interpreted as a value.
    Immediate,
    /// Parameter is interpreted as a position relative to the current `relative_base`
    Relative,
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
            2 => Ok(ParameterMode::Relative),
            _ => bail!("Unknown parameter mode `{}`", value),
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
enum OpCode {
    Binary {
        left: ParameterMode,
        right: ParameterMode,
        dest: ParameterMode,
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
    pub fn length(&self) -> i64 {
        match self {
            OpCode::Binary { .. } => 4,
            OpCode::Unary { .. } => 2,
            OpCode::Jump { .. } => 3,
            OpCode::Halt => 1,
        }
    }
}

impl TryFrom<i64> for OpCode {
    type Error = Error;

    fn try_from(n: i64) -> Result<Self> {
        // 01001 - first two digits are opcode, rest are parameter modes.
        // ---~~
        let (mut parameters, op) = (n / 100, (n % 100) as u8);

        match op {
            op if BinaryOperation::try_from(op).is_ok() => {
                let left_parameter_mode =
                    ParameterMode::try_from((parameters % 10) as u8).unwrap_or_default();

                parameters /= 10;

                let right_parameter_mode =
                    ParameterMode::try_from((parameters % 10) as u8).unwrap_or_default();

                parameters /= 10;

                let dest_parameter_mode = if parameters > 0 {
                    ParameterMode::try_from((parameters % 10) as u8)
                        .unwrap_or(ParameterMode::Immediate)
                } else {
                    ParameterMode::Immediate
                };

                Ok(OpCode::Binary {
                    left: left_parameter_mode,
                    right: right_parameter_mode,
                    dest: dest_parameter_mode,
                    t: op.try_into()?,
                })
            }
            op if UnaryOperation::try_from(op).is_ok() => {
                let parameter_mode =
                    ParameterMode::try_from((parameters % 10) as u8).unwrap_or_default();

                Ok(OpCode::Unary {
                    value: parameter_mode,
                    t: op.try_into()?,
                })
            }
            op if JumpOperation::try_from(op).is_ok() => {
                let left_parameter_mode =
                    ParameterMode::try_from((parameters % 10) as u8).unwrap_or_default();

                parameters /= 10;

                let right_parameter_mode =
                    ParameterMode::try_from((parameters % 10) as u8).unwrap_or_default();

                Ok(OpCode::Jump {
                    left: left_parameter_mode,
                    right: right_parameter_mode,
                    t: op.try_into()?,
                })
            }
            99 => Ok(OpCode::Halt),
            _ => bail!("`{:?}` is not a valid opcode.", n),
        }
    }
}

#[derive(Debug)]
pub struct IntcodeComputer {
    memory: Memory,
    io: Io,
    eip: i64,
    ebp: i64,
}

#[derive(Debug)]
pub enum ExecutionStatus {
    NeedInput,
    Done,
}

impl IntcodeComputer {
    pub fn new(mut program: Memory) -> Self {
        program.resize(1024 * 1024, 0);
        Self {
            memory: program,
            io: Io::new(),
            eip: 0,
            ebp: 0,
        }
    }

    pub fn parse_program(input: &str) -> Result<Memory> {
        input
            .trim_end()
            .split(",")
            .map(|number| {
                number
                    .parse::<i64>()
                    .with_context(|| format!("Expected a number `{}`", number))
            })
            .collect::<Result<Vec<i64>>>()
    }

    pub fn get(&self, i: Address) -> Result<i64> {
        if i < 0 {
            bail!("Cannot access memory at a negative offset `0x{:8x}`", i);
        }

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

    pub fn set_addr(&mut self, i: Address, value: i64) -> Result<()> {
        if i < 0 {
            bail!(
                "Cannot write to memory at a negative offset `0x{:8x} ({})`",
                i,
                i
            );
        }

        let stored = self.memory.get_mut(i as usize).with_context(|| {
            format!(
                "Out of bounds access while writing to memory at index `{}`",
                i
            )
        })?;

        *stored = value;
        Ok(())
    }

    pub fn write_to_input(&mut self, value: Vec<i64>) -> Result<()> {
        for i in value {
            self.io.input_write(i)?;
        }

        Ok(())
    }

    pub fn read_from_output(&mut self) -> Result<i64> {
        self.io.output_read()
    }

    pub fn into_output(self) -> VecDeque<i64> {
        self.io.into_output()
    }

    pub fn run(&mut self) -> Result<ExecutionStatus> {
        loop {
            // OpCode is always first two digits of number at `i`.
            let raw = self.get(self.eip)?;
            let op = OpCode::try_from(raw)?;
            debug!(
                "0x{:08x} ({:04}): `{:05}` => {:?} ",
                self.eip, self.eip, raw, &op
            );

            match &op {
                OpCode::Binary {
                    left,
                    right,
                    dest,
                    t,
                } => {
                    let (n1, n2, n3) = (
                        self.get(self.eip + 1)?,
                        self.get(self.eip + 2)?,
                        self.get(self.eip + 3)?,
                    );

                    let param1 = match left {
                        ParameterMode::Position => self.get(n1)?,
                        ParameterMode::Immediate => n1,
                        ParameterMode::Relative => self.get(n1 + self.ebp)?,
                    };

                    let param2 = match right {
                        ParameterMode::Position => self.get(n2)?,
                        ParameterMode::Immediate => n2,
                        ParameterMode::Relative => self.get(n2 + self.ebp)?,
                    };

                    let param3 = match dest {
                        ParameterMode::Position => self.get(n3)?,
                        ParameterMode::Immediate => n3,
                        ParameterMode::Relative => n3 + self.ebp,
                    };

                    match t {
                        BinaryOperation::Addition => {
                            let result = param1 + param2;
                            self.set_addr(param3, result)?;
                        }
                        BinaryOperation::Multiplication => {
                            let result = param1 * param2;
                            self.set_addr(param3, result)?;
                        }
                        BinaryOperation::Equals => {
                            if param1 == param2 {
                                self.set_addr(param3, 1)?;
                            } else {
                                self.set_addr(param3, 0)?;
                            }
                        }
                        BinaryOperation::LessThan => {
                            if param1 < param2 {
                                self.set_addr(param3, 1)?;
                            } else {
                                self.set_addr(param3, 0)?;
                            }
                        }
                    };
                    self.eip += op.length()
                }
                OpCode::Unary { value: v, t } => {
                    let dest = self.get(self.eip + 1)?;

                    let param1 = match v {
                        ParameterMode::Position => self.get(dest)?,
                        ParameterMode::Immediate => dest,
                        ParameterMode::Relative => self.get(dest + self.ebp)?,
                    };

                    match t {
                        UnaryOperation::Output => {
                            self.io.write(param1)?;
                        }
                        UnaryOperation::Store => match self.io.read() {
                            Err(_) => return Ok(ExecutionStatus::NeedInput),
                            Ok(i) => {
                                debug!("MEMSET: `0x{:08x}`={}", dest + self.ebp, i);
                                self.set_addr(dest + self.ebp, i)?;
                            }
                        },
                        UnaryOperation::AdjustRelativeBase => {
                            debug!("EBP: {} += {}", self.ebp, param1);
                            self.ebp += param1;
                        }
                    };
                    self.eip += op.length()
                }
                OpCode::Jump { left, right, t } => {
                    let (n1, n2) = (self.get(self.eip + 1)?, self.get(self.eip + 2)?);

                    let param1 = match left {
                        ParameterMode::Position => self.get(n1)?,
                        ParameterMode::Immediate => n1,
                        ParameterMode::Relative => self.get(n1 + self.ebp)?,
                    };

                    let param2 = match right {
                        ParameterMode::Position => self.get(n2)?,
                        ParameterMode::Immediate => n2,
                        ParameterMode::Relative => self.get(n2 + self.ebp)?,
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

                    if self.eip >= self.memory.len() as i64 {
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
    use crate::ParameterMode::{Immediate, Relative};

    macro_rules! test_memory {
        ($prog: expr, $i:expr, $o: expr) => {
            let bytecode = IntcodeComputer::parse_program($prog).unwrap();
            let mut computer = IntcodeComputer::new(bytecode);
            computer.write_to_input($i).unwrap();

            computer.run_until_halt().unwrap();
            assert!(computer.to_string().starts_with($o));
        };
    }

    macro_rules! test_output {
        ($prog: expr, $i:expr, $o: expr) => {
            env_logger::try_init().ok();
            let bytecode = IntcodeComputer::parse_program($prog).unwrap();
            let mut computer = IntcodeComputer::new(bytecode);
            computer.write_to_input($i).unwrap();

            computer.run_until_halt().unwrap();

            assert_eq!(computer.into_output(), $o);
        };
    }

    #[test]
    fn test_no_input() {
        test_memory!("1,0,0,0,99", vec![], "2,0,0,0,99");
        test_memory!("2,3,0,3,99", vec![], "2,3,0,6,99");
        test_memory!("2,4,4,5,99,0", vec![], "2,4,4,5,99,9801");
        test_memory!("1,1,1,4,99,5,6,0,99", vec![], "30,1,1,4,2,5,6,0,99");
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

    #[test]
    fn test_opcode() {
        assert_eq!(
            OpCode::try_from(21101).unwrap(),
            OpCode::Binary {
                left: Immediate,
                right: Immediate,
                dest: Relative,
                t: BinaryOperation::Addition
            }
        );
    }

    #[test]
    fn test_relative_mode() {
        test_output!(
            "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99",
            vec![],
            vec![109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99]
        );

        test_output!("104,1125899906842624,99", vec![], vec![1125899906842624]);

        test_output!(
            "1102,34915192,34915192,7,4,7,99,0",
            vec![],
            vec![1219070632396864]
        );
    }
}
