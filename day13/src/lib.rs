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

    'outer: loop {
        let input = match last_puck_position.0.cmp(&last_ball_position.0) {
            cmp::Ordering::Greater => vec![-1],
            cmp::Ordering::Less => vec![1],
            cmp::Ordering::Equal => vec![0],
        };

        if interactive {
            out.as_ref()
                .expect("Terminal exists when interactive")
                .clear_screen()?;
            out.as_ref()
                .expect("Terminal exists when interactive")
                .write_line(&format!("SCORE: {}", score))?;
        }

        game.write_to_input(input)
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
        let blocks_left = grid.iter().flatten().filter(|&&c| c == '*').count();

        if blocks_left == 0 {
            break 'outer;
        }

        if interactive {
            for line in grid.iter() {
                let line = String::from_iter(line.iter());
                out.as_ref()
                    .expect("Terminal exists when interactive")
                    .write_line(&line)?;
            }
        }
    }

    Ok(score)
}
