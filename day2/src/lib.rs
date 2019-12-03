use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

enum OpCode {
    Add,
    Multiply,
    Halt,
}

impl TryFrom<i32> for OpCode {
    type Error = Error;

    fn try_from(n: i32) -> Result<Self> {
        match n {
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Multiply),
            99 => Ok(OpCode::Halt),
            _ => bail!("`{}` is not a valid opcode."),
        }
    }
}

#[derive(Clone)]
struct ByteCodeInterpreter(Vec<i32>);

impl ByteCodeInterpreter {
    pub fn get(&self, i: i32) -> Result<i32> {
        self.0
            .get(i as usize)
            .with_context(|| format!("Out of bounds access on index `{}`", i))
            .map(|i| *i)
    }

    pub fn set(&mut self, i: i32, value: i32) -> Result<()> {
        let stored = self
            .0
            .get_mut(i as usize)
            .with_context(|| format!("Out of bounds access on index `{}`", i))?;

        *stored = value;
        Ok(())
    }

    /// In this program, the value placed in address 1 is called the noun, and the value placed in address 2 is called the verb.
    /// Each of the two input values will be between 0 and 99, inclusive.
    pub fn load_input(&mut self, noun: i32, verb: i32) -> Result<()> {
        self.set(1, noun)?;
        self.set(2, verb)?;

        Ok(())
    }

    /// Special input for part 1.
    pub fn restore_gravity_assist(&mut self) -> Result<()> {
        self.load_input(12, 2)
    }

    /// Output is stored at offset 0 after execution finishes.
    pub fn output(&self) -> Result<i32> {
        self.get(0)
    }

    pub fn run_until_halt(&mut self) -> Result<()> {
        let mut i = 0;

        loop {
            match OpCode::try_from(self.get(i)?)? {
                OpCode::Add => {
                    let (n1, n2, dest) = (self.get(i + 1)?, self.get(i + 2)?, self.get(i + 3)?);
                    self.set(dest, self.get(n1)? + self.get(n2)?)?;
                }
                OpCode::Multiply => {
                    let (n1, n2, dest) = (self.get(i + 1)?, self.get(i + 2)?, self.get(i + 3)?);
                    self.set(dest, self.get(n1)? * self.get(n2)?)?;
                }
                OpCode::Halt => break,
            }

            i += 4
        }

        Ok(())
    }
}

impl fmt::Display for ByteCodeInterpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = self.0.iter().join(",");
        f.write_str(&repr)?;

        Ok(())
    }
}

impl FromStr for ByteCodeInterpreter {
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

        Ok(ByteCodeInterpreter(inner))
    }
}

pub fn part_1(input: &str) -> Result<i32> {
    let mut computer = ByteCodeInterpreter::from_str(input)?;
    computer.restore_gravity_assist()?;
    computer.run_until_halt()?;
    computer.output()
}

pub fn part_2(input: &str) -> Result<i32> {
    let computer = ByteCodeInterpreter::from_str(input)?;

    for noun in 0..=99 {
        for verb in 0..=99 {
            // Clone computer here to avoid reparsing input.
            let mut attempt = computer.clone();
            attempt.load_input(noun, verb)?;
            attempt.run_until_halt()?;

            if attempt.output()? == 19690720 {
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
