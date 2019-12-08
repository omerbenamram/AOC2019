use anyhow::{bail, Context, Error, Result};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Write;

type PixelRow = Vec<Pixel>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
enum Pixel {
    Black,
    White,
    Transparent,
}

impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Pixel::Black => " ",
                Pixel::White => "0",
                Pixel::Transparent => " ",
            }
        )
    }
}

impl TryFrom<u8> for Pixel {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Pixel::White),
            0 => Ok(Pixel::Black),
            2 => Ok(Pixel::Transparent),
            _ => bail!("`{}` is not a valid pixel", value),
        }
    }
}

#[derive(Debug)]
struct Layer(Vec<PixelRow>);

#[derive(Debug)]
struct EncodedImage {
    layers: Vec<Layer>,
    x: usize,
    y: usize,
}

impl EncodedImage {
    /// X, Y are the dimensions of each layer in the image.
    pub fn with_dimensions(x: usize, y: usize, pixels: Vec<Pixel>) -> Result<EncodedImage> {
        let mut rows_iter = pixels.chunks_exact(x);
        let rows: Vec<PixelRow> = rows_iter.by_ref().map(|c| c.to_vec()).collect();

        if !rows_iter.remainder().is_empty() {
            return Err(Error::msg("Invalid input dimensions"));
        }

        let mut layers_iter = rows.chunks_exact(y);
        let layers: Vec<Layer> = layers_iter
            .by_ref()
            .map(|rows| Layer::new(rows.to_vec()))
            .collect();

        if !layers_iter.remainder().is_empty() {
            return Err(Error::msg("Invalid input dimensions"));
        }

        Ok(EncodedImage { layers, x, y })
    }

    pub fn checksum(&self) -> Result<usize> {
        let l = self
            .layers
            .iter()
            .min_by(|&l1, &l2| {
                l1.count_pixels(Pixel::Black)
                    .cmp(&l2.count_pixels(Pixel::Black))
            })
            .context("There isn't a definite minimum")?;

        Ok(l.count_pixels(Pixel::White) * l.count_pixels(Pixel::Transparent))
    }

    /// Decodes image by aligning layers.
    pub fn decode(&self) -> Layer {
        let mut decoded: Vec<PixelRow> = vec![];

        for r in 0..(self.y) {
            let mut row: PixelRow = vec![];
            for column in 0..(self.x) {
                let pixel = self
                    .layers
                    .iter()
                    .map(|l| l.pixel_at(r, column))
                    .find(|p| p.ne(&Pixel::Transparent))
                    .unwrap_or(Pixel::Transparent);

                row.push(pixel);
            }
            decoded.push(row)
        }
        Layer::new(decoded)
    }
}

impl Layer {
    pub fn new(pixels: Vec<PixelRow>) -> Self {
        Layer(pixels)
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Pixel {
        self.0[x][y]
    }

    pub fn count_pixels(&self, digit: Pixel) -> usize {
        self.iter_rows()
            .map(|row| row.iter().filter(|&&pixel| pixel == digit).count())
            .sum()
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = PixelRow> + '_ {
        let mut i = self.0.iter().cloned();
        std::iter::from_fn(move || i.next())
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.0.iter() {
            for &item in row {
                write!(f, "{}", item)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn parse_pixels(input: &str) -> Result<Vec<Pixel>> {
    input
        .trim()
        .chars()
        .map(|d| d.to_digit(10).expect("Invalid input"))
        .map(|n| Pixel::try_from(n as u8).context("Not a pixel"))
        .collect::<Result<Vec<Pixel>>>()
}

/// Find the layer that contains the fewest 0 digits.
/// On that layer, what is the number of 1 digits multiplied by the number of 2 digits?
pub fn part_1(input: &str) -> Result<usize> {
    let input = parse_pixels(input)?;

    let im = EncodedImage::with_dimensions(25, 6, input)?;

    im.checksum().context("Failed to calculate checksum")
}

pub fn part_2(input: &str) -> Result<String> {
    let input = parse_pixels(input)?;
    let im = EncodedImage::with_dimensions(25, 6, input)?;

    let mut result = String::with_capacity(25 * 10);

    writeln!(
        result,
        "\n
--------------------------------

{}
--------------------------------
    ",
        im.decode()
    )?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let pixels = parse_pixels("121111222012").unwrap();
        let im = EncodedImage::with_dimensions(3, 2, pixels).unwrap();
        assert_eq!(im.checksum().unwrap(), 5);
    }
}
