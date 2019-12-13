#![deny(unused_must_use)]

use anyhow::{bail, Context, Result};
use console::Term;
use intcode_computer::{ExecutionStatus, IntcodeComputer};
use std::cmp;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::thread;
use std::time::Duration;

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

pub fn part_2(input: &str, interactive: bool) -> Result<i64> {
    let program = IntcodeComputer::parse_program(&input)?;
    let mut game = IntcodeComputer::new(program);
    let mut score = 0;
    // PLAY FOR FREE
    game.set_addr(0, 2)?;

    let mut grid = vec![vec![' '; 50]; 40];

    let out = if interactive {
        let t = Term::stdout();
        assert!(t.is_term());
        Some(t)
    } else {
        None
    };

    let mut last_puck_position = (0, 0);
    let mut last_ball_position = (0, 0);
    let mut blocks_left = None;
    let mut input = [0_i64; 1];
    let mut output = Vec::with_capacity(100);

    'outer: loop {
        input[0] = match last_puck_position.0.cmp(&last_ball_position.0) {
            cmp::Ordering::Greater => -1,
            cmp::Ordering::Less => 1,
            cmp::Ordering::Equal => 0,
        };

        if interactive {
            out.as_ref()
                .expect("Terminal exists when interactive")
                .clear_screen()?;
            out.as_ref()
                .expect("Terminal exists when interactive")
                .write_line(&format!("SCORE: {}", score))?;
        }

        game.write_to_input(&input)
            .context("Failed to write to input")
            .unwrap();

        'inner: loop {
            let status = game.step().unwrap();
            match status {
                ExecutionStatus::NeedInput => break 'inner,
                ExecutionStatus::Halted => break 'inner,
                ExecutionStatus::Done => {}
            }
        }

        while let Ok(i) = game.read_from_output() {
            output.push(i)
        }

        let mut tiles = vec![];

        for tile in output.chunks_exact(3) {
            let (x, y, t) = (tile[0] as isize, tile[1] as isize, tile[2]);
            tiles.push(((x, y), Tile::try_from(t).unwrap()));
        }

        if blocks_left.is_none() {
            let count = tiles.iter().filter(|(_, t)| *t == Tile::Block).count();
            blocks_left = Some(count);
        }

        for tile in tiles.iter() {
            let ((x, y), t) = (tile.0, tile.1);

            if ((x, y)) == ((-1, 0)) {
                if let Tile::SegmentDisplay(s) = t {
                    score = s
                }
                continue;
            }

            if let Tile::Empty = t {
                if grid[y as usize][x as usize] == '*' {
                    blocks_left = Some(blocks_left.as_ref().unwrap().saturating_sub(1));
                }
            }

            if let Tile::Ball = t {
                last_ball_position = (x, y);
            }

            if let Tile::HorizontalPaddle = t {
                if last_ball_position == (x, y) {
                    bail!("Game over");
                }
                last_puck_position = (x, y);
            }

            // It's only safe to cast now since -1 positions have been dealt with
            grid[y as usize][x as usize] = match t {
                Tile::Empty => ' ',
                Tile::Wall => '^',
                Tile::Block => '*',
                Tile::HorizontalPaddle => '_',
                Tile::Ball => '@',
                Tile::SegmentDisplay(_) => unreachable!(),
            };
        }

        output.truncate(0);

        if *blocks_left.as_ref().unwrap() == 0 {
            break 'outer;
        }

        if interactive {
            for line in grid.iter() {
                let line = String::from_iter(line.iter());
                out.as_ref()
                    .expect("Terminal exists when interactive")
                    .write_line(&line)?;
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(score)
}
