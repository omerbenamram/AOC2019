use anyhow::{bail, Context, Error, Result};
use std::ops::RangeInclusive;

fn parse_range(input: &str) -> Result<RangeInclusive<i32>> {
    let range: Vec<i32> = input
        .trim()
        .split("-")
        .map(|num| {
            num.parse()
                .context(format!("Failed to parse input `{}`", num))
        })
        .collect::<Result<Vec<i32>>>()?;

    if range.len() != 2 {
        bail!("Expected exactly two numbers.");
    }

    Ok(range[0]..=range[1])
}

pub fn part_1(input: &str) -> Result<i32> {
    let range = parse_range(input)?;
    let mut count = 0;
    for n in range {
        if check_number(n) {
            count += 1;
        }
    }
    Ok(count)
}

pub fn part_2(input: &str) -> Result<i32> {
    let range = parse_range(input)?;
    let mut count = 0;
    for n in range {
        if check_number_updated(n) {
            count += 1;
        }
    }
    Ok(count)
}

fn check_number_updated(n: i32) -> bool {
    let mut digits_seen = [false; 10];
    let mut following_digits_seen = false;
    let mut digit_group_size = 1;
    let mut previous_digit = None;

    for digit in n.to_string().chars() {
        let digit = digit.to_digit(10).expect("This can only be a digit.");

        // Check following digits
        if let Some(prev) = previous_digit.take() {
            if prev == digit {
                digit_group_size += 1;
            } else {
                // seen a new number, check if condition was met.
                if digit_group_size == 2 {
                    following_digits_seen = true
                }
                digit_group_size = 1;
            }
        }

        // Check if digit is smallest we've seen so far.
        let digits_so_far_are_increasing = digits_seen.iter().enumerate().all(|(d, was_seen)| {
            if *was_seen && (d > digit as usize) {
                false
            } else {
                true
            }
        });

        if !digits_so_far_are_increasing {
            return false;
        }

        digits_seen[digit as usize] = true;
        previous_digit = Some(digit);
    }

    // If last group was 2, it's still OK.
    if following_digits_seen || digit_group_size == 2 {
        true
    } else {
        false
    }
}

fn check_number(n: i32) -> bool {
    let mut digits_seen = [false; 10];
    let mut following_digits_seen = false;
    let mut previous_digit = None;

    for digit in n.to_string().chars() {
        let digit = digit.to_digit(10).expect("This can only be a digit.");

        // Check following digits
        if let Some(prev) = previous_digit.take() {
            if prev == digit {
                following_digits_seen = true
            }
        }

        // Check if digit is smallest we've seen so far.
        let digits_so_far_are_increasing = digits_seen.iter().enumerate().all(|(d, was_seen)| {
            if *was_seen && (d > digit as usize) {
                false
            } else {
                true
            }
        });

        if !digits_so_far_are_increasing {
            return false;
        }

        digits_seen[digit as usize] = true;
        previous_digit = Some(digit);
    }

    if following_digits_seen {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_part_1() {
        assert_eq!(check_number(111111), true);
        assert_eq!(check_number(223450), false);
        assert_eq!(check_number(123789), false);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(check_number_updated(111111), false, "111111");
        assert_eq!(check_number_updated(123444), false, "123444");
        assert_eq!(check_number_updated(111122), true, "111122");
    }
}
