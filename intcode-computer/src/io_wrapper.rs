use crate::Memory;
use anyhow::{Error, Result};
use std::collections::VecDeque;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct Io {
    input: VecDeque<i32>,
    output: VecDeque<i32>,
}

impl Io {
    pub fn new() -> Self {
        Io {
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }

    /// Consumes self, returning the resulting IO.
    pub fn into_output(self) -> VecDeque<i32> {
        self.output
    }

    /// Read from input
    pub fn output_read(&mut self) -> Result<i32, Error> {
        self.output.pop_front().ok_or(Error::msg("EOF"))
    }

    /// Read from input
    pub fn input_write(&mut self, value: i32) -> Result<(), Error> {
        self.input.push_back(value);

        Ok(())
    }

    /// Read from input
    pub fn read(&mut self) -> Result<i32, Error> {
        self.input.pop_front().ok_or(Error::msg("EOF"))
    }

    /// Write to output
    pub fn write(&mut self, value: i32) -> Result<(), Error> {
        self.output.push_back(value);

        Ok(())
    }
}
