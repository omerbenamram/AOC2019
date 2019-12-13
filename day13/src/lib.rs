#![deny(unused_must_use)]

use anyhow::Error;
use anyhow::{bail, Context, Result};
use console::{Key, Term};
use intcode_computer::{ExecutionStatus, IntcodeComputer};
use itertools::Itertools;
use log::debug;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{stdin, Read, Write};
use std::iter::FromIterator;
use std::ptr::write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use std::{io, thread};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
enum Tile {
    // empty tile. No game object appears in this tile
    Empty,
    // a wall tile. Walls are indestructible barriers.
    Wall,
    //  a block tile. Blocks can be broken by the ball.
    Block,
    // a horizontal paddle tile. The paddle is indestructible.
    HorizontalPaddle,
    // a ball tile. The ball moves diagonally and bounces off objects.
    Ball,
    SegmentDisplay(i64),
}

impl From<i64> for Tile {
    fn from(value: i64) -> Self {
        match value {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::HorizontalPaddle,
            4 => Tile::Ball,
            _ => Tile::SegmentDisplay(value),
        }
    }
}

type Coord = (isize, isize);

pub fn part_1(input: &str) -> Result<usize> {
    let program = IntcodeComputer::parse_program(input)?;
    let mut screen = IntcodeComputer::new(program);
    screen.run_until_halt()?;

    let output = screen.into_output();
    let output = Vec::from(output);

    let mut tiles = vec![];
    for tile in output.chunks_exact(3) {
        let (x, y, t) = (tile[0], tile[1], tile[2]);
        tiles.push(((x, y), Tile::try_from(t)?));
    }

    Ok(tiles.iter().filter(|(_, t)| *t == Tile::Block).count())
}

pub fn part_2(input: &str) -> Result<()> {
    let mut program = IntcodeComputer::parse_program(input)?;
    let mut game = IntcodeComputer::new(program);
    let mut score = 0;
    // PLAY FOR FREE
    game.set_addr(0, 2)?;

    let mut grid = [[' '; 50]; 40];

    let out = Term::stdout();
    assert!(out.is_term());

    let (send, rvc) = channel();

    let t_out = out.clone();

    let t = std::thread::spawn(move || 'outer: loop {
        t_out.clear_screen().unwrap();
        t_out.write_line(&format!("SCORE: {}", score)).unwrap();

        let mut v = vec![0];

        let v = if let Ok(k) = rvc.try_recv() {
            let inner = match k {
                Key::ArrowLeft => -1,
                Key::ArrowRight => 1,
                _ => 0,
            };
            vec![inner]
        } else {
            vec![0]
        };

        game.write_to_input(v)
            .context("Failed to write to input")
            .unwrap();

        'inner: loop {
            let status = game.step().unwrap();
            match status {
                ExecutionStatus::NeedInput => break,
                ExecutionStatus::Halted => break,
                ExecutionStatus::Done => {}
            }
        }

        let mut output = vec![];

        while let Ok(i) = game.read_from_output() {
            output.push(i)
        }

        let mut tiles = vec![];

        for tile in output.chunks_exact(3) {
            let (x, y, t) = (tile[0] as isize, tile[1] as isize, tile[2]);
            tiles.push(((x, y), Tile::try_from(t).unwrap()));
        }

        for tile in tiles.iter() {
            let ((x, y), t) = (tile.0, tile.1);

            if ((x, y)) == ((-1, 0)) {
                if let Tile::SegmentDisplay(s) = t {
                    score = s
                }
                continue;
            }

            // It's only safe to cast now since -1 positions have been dealt with
            grid[y as usize][x as usize] = match t {
                Tile::Empty => ' ',
                Tile::Wall => '^',
                Tile::Block => '*',
                Tile::HorizontalPaddle => '_',
                Tile::Ball => '@',
                Tile::SegmentDisplay(_) => unreachable!(),
            }
        }

        for line in grid.iter() {
            let line = String::from_iter(line.iter());
            t_out.write_line(&line).unwrap();
        }

        t_out.flush().unwrap();

        thread::sleep(Duration::from_millis(100))
    });

    'io: loop {
        send.send(out.read_key()?)?
    }
    Ok(())
}
