#![allow(explicit_defaults, no_unwrap, unbounded_loop)]

use std::collections::BTreeMap;
use std::collections::HashMap;

fn keep_going() -> bool {
    true
}

fn next_byte() -> u8 {
    1
}

fn push_without_bound() {
    let mut items = Vec::new();
    loop {
        items.push(next_byte());
    }
}

fn insert_without_bound() {
    let mut items: HashMap<u8, u8> = Default::default();
    while keep_going() {
        items.insert(next_byte(), next_byte());
    }
}

fn extend_without_bound() {
    let mut items = Vec::new();
    while keep_going() {
        items.extend([next_byte()]);
    }
}

fn push_in_for_without_bound() {
    let mut items = Vec::new();
    for value in 0..10u8 {
        items.push(value);
    }
}

fn with_capacity_is_allowed() {
    let mut items = Vec::with_capacity(16);
    loop {
        items.push(next_byte());
    }
}

fn reserve_exact_is_allowed() {
    let mut items = Vec::new();
    items.reserve_exact(16);
    while keep_going() {
        items.push(next_byte());
    }
}

fn try_reserve_is_allowed() {
    let mut items = Vec::new();
    items.try_reserve(16).expect("reserve works");
    while keep_going() {
        items.push(next_byte());
    }
}

fn guarded_by_len_is_allowed(limit: usize) {
    let mut items: HashMap<u8, u8> = Default::default();
    while items.len() < limit {
        items.insert(next_byte(), next_byte());
    }
}

fn guarded_by_break_is_allowed(limit: usize) {
    let mut items = Vec::new();
    loop {
        if items.len() >= limit {
            break;
        }
        items.push(next_byte());
    }
}

fn outer_binding_is_ignored() {
    let mut items = BTreeMap::new();
    {
        while keep_going() {
            items.insert(next_byte(), next_byte());
        }
    }
}

const REQUIRED_ITEMS: [u8; 3] = [1, 2, 3];
const MAX_ITEMS: usize = 16;
const ZERO_ITEMS: usize = 0;
const VACUOUS_MAX: usize = usize::MAX;

fn fixed_array_for_is_bounded() {
    let mut items = Vec::new();
    for value in REQUIRED_ITEMS {
        items.push(value);
    }
}

fn guarded_slice_for_is_bounded(input: &[u8]) {
    let mut items = Vec::new();
    if input.len() > MAX_ITEMS {
        let _ = next_byte();
        return;
    }
    for &value in input {
        items.push(value);
    }
}

fn zero_bound_slice_is_bounded(input: &[u8]) {
    let mut items = Vec::new();
    if input.len() > ZERO_ITEMS {
        return;
    }
    for &value in input {
        items.push(value);
    }
}

fn vacuous_max_guard_warns(input: &[u8]) {
    let mut items = Vec::new();
    if input.len() > VACUOUS_MAX {
        return;
    }
    for &value in input {
        items.push(value);
    }
}

fn unguarded_slice_for_warns(input: &[u8]) {
    let mut items = Vec::new();
    for &value in input {
        items.push(value);
    }
}

fn non_exiting_slice_guard_warns(input: &[u8]) {
    let mut items = Vec::new();
    if input.len() > MAX_ITEMS {
        let _ = next_byte();
    }
    for &value in input {
        items.push(value);
    }
}

fn guard_on_other_slice_warns(input: &[u8], other: &[u16]) {
    let mut items = Vec::new();
    if other.len() > MAX_ITEMS {
        return;
    }
    for &value in input {
        items.push(value);
    }
}

fn nested_dynamic_for_warns(input: &[u8]) {
    let mut items = Vec::new();
    for _ in REQUIRED_ITEMS {
        for &value in input {
            items.push(value);
        }
    }
}

fn fixed_array_extend_can_still_grow(input: &[u8]) {
    let mut items = Vec::new();
    for _ in REQUIRED_ITEMS {
        items.extend(input.iter().copied());
    }
}

fn mutable_slice_source_can_rebind(input: &[u8]) {
    let mut items = Vec::new();
    let mut source: &[u8] = &[];
    if source.len() > MAX_ITEMS {
        return;
    }
    source = input;
    for &value in source {
        items.push(value);
    }
}

fn main() {}
