use anyhow::{Context, Result};

/// Fuel required to launch a given module is based on its mass.
/// Specifically, to find the fuel required for a module, take its mass, divide by three, round down, and subtract 2.
/// Any mass that would require negative fuel should instead be treated as if it requires zero fuel;
fn calculate_fuel(module_mass: usize) -> usize {
    (module_mass / 3).saturating_sub(2)
}

/// Fuel itself requires fuel just like a module - take its mass, divide by three, round down, and subtract 2.
/// However, that fuel also requires fuel, and that fuel requires fuel, and so on.
/// the remaining mass, if any, is instead handled by wishing really hard, which has no mass and is outside the scope of this calculation.
fn calculate_fuel_recursively(module_mass: usize) -> usize {
    let mut total_fuel = 0;

    let mut fuel = calculate_fuel(module_mass);

    while fuel > 0 {
        total_fuel += fuel;
        fuel = calculate_fuel(fuel)
    }

    total_fuel
}

fn fuel_for_modules(input: &str, fuel_calculator: impl Fn(usize) -> usize + Copy) -> Result<usize> {
    input
        .lines()
        .map(|line| {
            line.parse::<usize>()
                .with_context(|| format!("Failed to convert `{}` to a numerical value.", line))
                .map(fuel_calculator)
        })
        .sum()
}

/// What is the sum of fuel required for all of your modules?
pub fn part_1(input: &str) -> Result<usize> {
    fuel_for_modules(input, calculate_fuel)
}

/// What is the sum of fuel required for all of your modules?
pub fn part_2(input: &str) -> Result<usize> {
    fuel_for_modules(input, calculate_fuel_recursively)
}

#[test]
fn test_calculate_fuel() {
    assert_eq!(calculate_fuel(12), 2);
    assert_eq!(calculate_fuel(14), 2);
    assert_eq!(calculate_fuel(1969), 654);
    assert_eq!(calculate_fuel(100_756), 33583);
}

#[test]
fn test_calculate_fuel_recursively() {
    assert_eq!(calculate_fuel_recursively(100_756), 50346);
}
