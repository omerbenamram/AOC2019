use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use log::{debug, trace};
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

impl BinaryOperation {
    pub fn eval(&self, left: i64, right: i64) -> i64 {
        match self {
            BinaryOperation::Addition => left + right,
            BinaryOperation::Multiplication => left * right,
            BinaryOperation::Equals => (left == right) as i64,
            BinaryOperation::LessThan => (left < right) as i64,
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
enum JumpOperation {
    JumpIfTrue,
    JumpIfFalse,
}

#[derive(Debug, PartialOrd, PartialEq)]
enum Parameter {
    /// Causes the parameter to be interpreted as a position.
    Position(Address),
    /// Causes the parameter to be interpreted as a value.
    Immediate(i64),
    /// Parameter is interpreted as a position relative to the current `relative_base`
    Relative(Address),
}

#[derive(Debug, PartialOrd, PartialEq)]
enum OpCode {
    Binary {
        left: Parameter,
        right: Parameter,
        dest: Address,
        t: BinaryOperation,
    },
    /// Takes a single integer as input and saves it to the position given by its only parameter.
    /// For example, the instruction 3,50 would take an input value and store it at address 50
    Input {
        address: Address,
    },
    /// Outputs the value of its only parameter.
    /// For example, the instruction 4,50 would output the value at address 50
    Output {
        value: Parameter,
    },
    /// Adjusts the relative base by the value of its only parameter.
    AdjustRelativeBase {
        value: Parameter,
    },
    Jump {
        condition: Parameter,
        right: Parameter,
        t: JumpOperation,
    },
    Halt,
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

    pub fn from_program_without_extra_memory(program: Memory) -> Self {
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
            bail!("Cannot access memory at a negative offset `0x{:08x}`", i);
        }

        self.memory
            .get(i as usize)
            .with_context(|| {
                format!(
                    "Out of bounds access while reading from memory at offset `0x{:08x} ({})`",
                    i, i
                )
            })
            .map(|i| *i)
    }

    pub fn set_addr(&mut self, i: Address, value: i64) -> Result<()> {
        if i < 0 {
            bail!(
                "Cannot write to memory at a negative offset `0x{:08x} ({})`",
                i,
                i
            );
        }

        let stored = self.memory.get_mut(i as usize).with_context(|| {
            format!(
                "Out of bounds access while writing to memory at offset `0x{:08x} ({})`",
                i, i
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

    fn load(&self, parameter: &Parameter) -> Result<i64> {
        let p = match parameter {
            Parameter::Position(i) => self.get(*i)?,
            Parameter::Immediate(i) => *i,
            Parameter::Relative(i) => self.get(i + self.ebp)?,
        };

        Ok(p)
    }

    fn read_opcode(&mut self) -> Result<OpCode> {
        let raw = self.get(self.eip)?;

        // 01001 - first two digits are opcode, rest are parameter modes.
        // ---~~
        let (mut parameters, op) = (raw / 100, (raw % 100) as u8);

        // inspired by https://github.com/matklad/aoc2019/blob/master/src/lib.rs#L158
        // Arugment can either be a value or an address.
        // Input to this macro is in the form of `let (val1, val2, dest) = parse_arguments!(v v a)`.
        macro_rules! parse_arguments {
            ($($m:ident)*) => {{
                // https://doc.rust-lang.org/rust-by-example/macros/repeat.html
                //          Unpack a tuple of `parse_arguments`
                //          for each ident
                let args = ($(parse_arguments!(@ $m),)*);
                if parameters != 0 {
                    bail!("Did not consume all parameters.");
                }
                self.eip += 1;
                args
            }};
            // Parses a `value`
            (@ v) => {{
                self.eip += 1;

                let parameter_value = self.get(self.eip)?;
                let parameter_mode = parameters % 10;

                let actual_value = match parameter_mode {
                    // Position - read the value at the address.
                    0 => Parameter::Position(parameter_value),
                    // Immediate - just use the value.
                    1 => Parameter::Immediate(parameter_value),
                    // Relative - use position + relative base.
                    2 => Parameter::Relative(parameter_value),
                    _ => bail!("Invalid parameter mode `{}`", parameter_mode)
                };

                parameters /= 10;
                actual_value
            }};
            // Parses an address, addresses are always an i64 (`Address`)
            (@ a) => {{
                self.eip += 1;
                let parameter_value = self.get(self.eip)?;
                let parameter_mode = parameters % 10;

                let actual_value = match parameter_mode {
                    // Addresses cannot be positional, 0 will be given no mode is specified,
                    // in this case we use the address value.
                    0 | 1 =>  parameter_value,
                    // Relative - use position + relative base.
                    2 => parameter_value + self.ebp,
                    _ => bail!("Invalid parameter mode `{}`", parameter_mode)
                };

                parameters /= 10;
                actual_value
            }}
        }

        let op = match op {
            // Binary ops
            1 | 2 | 7 | 8 => {
                let (left, right, dest) = parse_arguments!(v v a);
                let t = match op {
                    1 => BinaryOperation::Addition,
                    2 => BinaryOperation::Multiplication,
                    7 => BinaryOperation::LessThan,
                    8 => BinaryOperation::Equals,
                    _ => unreachable!(),
                };

                OpCode::Binary {
                    left,
                    right,
                    dest,
                    t,
                }
            }
            3 => {
                let (dst,) = parse_arguments!(a);
                OpCode::Input { address: dst }
            }
            4 => {
                let (value,) = parse_arguments!(v);
                OpCode::Output { value }
            }
            5 | 6 => {
                let (cond, dest) = parse_arguments!(v v);
                let t = match op {
                    5 => JumpOperation::JumpIfTrue,
                    6 => JumpOperation::JumpIfFalse,
                    _ => unreachable!(),
                };

                OpCode::Jump {
                    condition: cond,
                    right: dest,
                    t,
                }
            }
            9 => {
                let (value,) = parse_arguments!(v);
                OpCode::AdjustRelativeBase { value }
            }
            99 => OpCode::Halt,
            _ => bail!("Unknown opcode `{}`", raw),
        };

        debug!(
            "0x{:08x} ({:04}): `{:05}` => {:?} ",
            self.eip, self.eip, raw, &op
        );

        Ok(op)
    }

    pub fn run(&mut self) -> Result<ExecutionStatus> {
        loop {
            let op = self.read_opcode()?;

            match &op {
                OpCode::Binary {
                    left,
                    right,
                    dest,
                    t,
                } => {
                    let left = self.load(left)?;
                    let right = self.load(right)?;
                    let result = t.eval(left, right);
                    self.set_addr(*dest, result)?;
                }
                OpCode::Input { address } => match self.io.read() {
                    Err(_) => return Ok(ExecutionStatus::NeedInput),
                    Ok(i) => {
                        trace!("MEMSET: `0x{:08x}`={}", address, i);
                        self.set_addr(*address, i)?;
                    }
                },
                OpCode::Output { value } => {
                    self.io.write(self.load(value)?)?;
                }
                OpCode::AdjustRelativeBase { value } => {
                    let value = self.load(value)?;
                    trace!("EBP: {} += {}", self.ebp, value);
                    self.ebp += value;
                }
                OpCode::Jump {
                    condition: left,
                    right,
                    t,
                } => {
                    let condition = self.load(left)?;
                    let right = self.load(right)?;

                    match t {
                        JumpOperation::JumpIfTrue => {
                            if condition != 0 {
                                self.eip = right
                            }
                        }
                        JumpOperation::JumpIfFalse => {
                            if condition == 0 {
                                self.eip = right
                            }
                        }
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
    use crate::Parameter::{Immediate, Relative};

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
