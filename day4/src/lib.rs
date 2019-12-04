use anyhow::{bail, Context, Error, Result};

type Password = [u8; 6];

pub fn parse_range(input: &str) -> Result<(u32, u32)> {
    let range: Vec<u32> = input
        .trim()
        .split("-")
        .map(|num| {
            num.parse()
                .context(format!("Failed to parse input `{}`", num))
        })
        .collect::<Result<Vec<u32>>>()?;

    if range.len() != 2 {
        bail!("Expected exactly two numbers.");
    }

    Ok((range[0], range[1]))
}

pub fn part_1(low: u32, hi: u32) -> u32 {
    (low..=hi)
        .map(num_to_arr)
        .filter(|&n| check_password(n))
        .count() as u32
}

pub fn part_2(low: u32, hi: u32) -> u32 {
    (low..=hi)
        .map(num_to_arr)
        .filter(|&n| check_password_2(n))
        .count() as u32
}

fn num_to_arr(mut n: u32) -> Password {
    let mut res: Password = [0; 6];
    for digit in res.iter_mut().rev() {
        *digit = (n % 10) as u8;
        n /= 10;
    }
    res
}

fn check_password_2(pass: Password) -> bool {
    let mut largest_digit_seen = 0;
    let mut following_digits_seen = false;
    let mut digit_group_size = 1;
    let mut previous_digit = None;

    for &digit in pass.iter() {
        if digit >= largest_digit_seen {
            largest_digit_seen = digit;
        } else {
            return false;
        }

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

        previous_digit = Some(digit);
    }

    // If last group was 2, it's still OK.
    if following_digits_seen || digit_group_size == 2 {
        true
    } else {
        false
    }
}

fn check_password(pass: Password) -> bool {
    let mut largest_digit_seen = 0;
    let mut following_digits_seen = false;
    let mut previous_digit = None;

    for &digit in pass.iter() {
        if digit >= largest_digit_seen {
            largest_digit_seen = digit;
        } else {
            return false;
        }

        // Check following digits
        if let Some(prev) = previous_digit.take() {
            if prev == digit {
                following_digits_seen = true
            }
        }

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
        assert_eq!(check_password([1, 1, 1, 1, 1, 1]), true);
        assert_eq!(check_password([2, 2, 3, 4, 5, 0]), false);
        assert_eq!(check_password([1, 2, 3, 7, 8, 9]), false);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(check_password_2([1, 1, 1, 1, 1, 1]), false, "111111");
        assert_eq!(check_password_2([1, 2, 3, 4, 4, 4]), false, "123444");
        assert_eq!(check_password_2([1, 1, 1, 1, 2, 2]), true, "111122");
    }
}
