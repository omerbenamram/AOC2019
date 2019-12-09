use anyhow::{Error, Result};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Io {
    input: VecDeque<i64>,
    output: VecDeque<i64>,
}

impl Io {
    pub fn new() -> Self {
        Io {
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }

    /// Consumes self, returning the resulting IO.
    pub fn into_output(self) -> VecDeque<i64> {
        self.output
    }

    /// Read from input
    pub fn output_read(&mut self) -> Result<i64, Error> {
        self.output.pop_front().ok_or_else(|| Error::msg("EOF"))
    }

    /// Read from input
    pub fn input_write(&mut self, value: i64) -> Result<(), Error> {
        self.input.push_back(value);

        Ok(())
    }

    /// Read from input
    pub fn read(&mut self) -> Result<i64, Error> {
        self.input.pop_front().ok_or(Error::msg("EOF"))
    }

    /// Write to output
    pub fn write(&mut self, value: i64) -> Result<(), Error> {
        self.output.push_back(value);

        Ok(())
    }
}
