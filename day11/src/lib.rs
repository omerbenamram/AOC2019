use anyhow::Error;
#[deny(unused_must_use)]
use anyhow::{bail, Context, Result};
use intcode_computer::{ExecutionStatus, IntcodeComputer};
use log::debug;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
enum Color {
    Black,
    White,
}

impl TryFrom<i64> for Color {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        let res = match value {
            0 => Color::Black,
            1 => Color::White,
            _ => bail!("Not a color `{}`", value),
        };
        Ok(res)
    }
}

impl From<Color> for i64 {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}

pub fn part_1(input: &str) -> Result<usize> {
    let program = IntcodeComputer::parse_program(input)?;
    let mut robot = IntcodeComputer::new(program);

    let tiles = tiles(&mut robot, Color::Black)?;
    Ok(tiles.len())
}

type Coord = (i32, i32);

fn tiles(robot: &mut IntcodeComputer, start_color: Color) -> Result<HashMap<Coord, Color>> {
    let mut position = (0, 0);
    let mut robot_direction = Direction::Up;
    let mut visited_tiles = HashMap::new();
    visited_tiles.insert(position, start_color);

    'outer: loop {
        let panel = visited_tiles.entry(position).or_insert(Color::Black);
        debug!("Panel {:?} is {:?}", position, panel);
        robot.write_to_input(vec![panel.clone().into()])?;

        'inner: loop {
            match robot.step()? {
                ExecutionStatus::NeedInput => break 'inner,
                ExecutionStatus::Done => {}
                ExecutionStatus::Halted => break 'outer,
            }
        }

        let new_panel_color = Color::try_from(
            robot
                .read_from_output()
                .context("Expected robot to provide new panel color")?,
        )?;

        let turn = robot
            .read_from_output()
            .context("Expected robot to provide a new direction")?;

        if !robot.read_from_output().is_err() {
            bail!("Too much output")
        }

        // Paint panel
        debug!("PAINTING {:?}, {:?}", position, new_panel_color);
        *panel = new_panel_color;

        robot_direction = match (robot_direction, turn) {
            // turn left
            (Direction::Up, 0) => Direction::Left,
            (Direction::Left, 0) => Direction::Down,
            (Direction::Down, 0) => Direction::Right,
            (Direction::Right, 0) => Direction::Up,
            // turn right
            (Direction::Up, 1) => Direction::Right,
            (Direction::Right, 1) => Direction::Down,
            (Direction::Down, 1) => Direction::Left,
            (Direction::Left, 1) => Direction::Up,
            _ => bail!("Invalid turn `{}`", turn),
        };

        position = match robot_direction {
            Direction::Up => (position.0, position.1 + 1),
            Direction::Down => (position.0, position.1 - 1),
            Direction::Left => (position.0 - 1, position.1),
            Direction::Right => (position.0 + 1, position.1),
        };
    }

    Ok(visited_tiles)
}

pub fn part_2(input: &str) -> Result<()> {
    let program = IntcodeComputer::parse_program(input)?;
    let mut robot = IntcodeComputer::new(program);

    let tiles = tiles(&mut robot, Color::White)?;

    for i in -5..10 {
        for j in -20..60 {
            if j % 10 == 0 {
                print!(" ")
            }
            match tiles.get(&(j, -1 * i)) {
                Some(Color::White) => print!("#"),
                Some(Color::Black) => print!("."),
                _ => print!("."),
            }
        }
        println!()
    }

    Ok(())
}
