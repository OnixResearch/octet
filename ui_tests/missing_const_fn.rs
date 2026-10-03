#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(tigerstyle::missing_const_fn)]

use std::fs as disk;
use std::path::Path;

const SCALE_FACTOR: u64 = 2;

fn scale_value(value: u64) -> u64 {
    value * SCALE_FACTOR
}

fn branch_value(value: u64) -> u64 {
    if value == 0 {
        SCALE_FACTOR
    } else {
        value + 1
    }
}

fn match_value(value: u64) -> u64 {
    match value {
        0 => SCALE_FACTOR,
        _ => value * SCALE_FACTOR,
    }
}

fn load_size(path: &Path) -> usize {
    disk::read(path).map_or(0, |contents| contents.len()) + 1
}

fn path_exists(path: &Path) -> bool {
    path.exists() && 1 + 1 == 2
}

struct Repository;

impl Repository {
    fn fetch(&self) -> u64 {
        1
    }

    fn inherent_helper(&self, value: u64) -> u64 {
        value + 1
    }
}

trait Arithmetic {
    fn default_helper(value: u64) -> u64 {
        value + 1
    }
}

fn repository_value(repository: &Repository) -> u64 {
    repository.fetch() + 1
}

fn checked_conversion(value: u64) -> Result<usize, std::num::TryFromIntError> {
    usize::try_from(value + 1)
}

fn callback_value(value: u64, callback: fn(u64) -> u64) -> u64 {
    callback(value) + 1
}

fn borrowed_value(value: &u64) -> u64 {
    *value + 1
}

fn mutated_value(value: u64) -> u64 {
    let mut result = value;
    result += 1;
    result
}

pub fn public_scale(value: u64) -> u64 {
    value * SCALE_FACTOR
}

pub(crate) fn crate_scale(value: u64) -> u64 {
    value + SCALE_FACTOR
}

fn main() {
    let _scaled = (
        scale_value(SCALE_FACTOR),
        branch_value(SCALE_FACTOR),
        match_value(SCALE_FACTOR),
    );
    let _ = load_size as fn(&Path) -> usize;
    let _ = path_exists as fn(&Path) -> bool;
    let _ = repository_value as fn(&Repository) -> u64;
    let _ = checked_conversion as fn(u64) -> Result<usize, std::num::TryFromIntError>;
    let _ = callback_value as fn(u64, fn(u64) -> u64) -> u64;
    let _ = borrowed_value as fn(&u64) -> u64;
    let _ = mutated_value as fn(u64) -> u64;
    let _ = public_scale as fn(u64) -> u64;
    let _ = crate_scale as fn(u64) -> u64;
    let _ = Repository.inherent_helper(SCALE_FACTOR);
    let _ = <Repository as Arithmetic>::default_helper(SCALE_FACTOR);
}

impl Arithmetic for Repository {}
