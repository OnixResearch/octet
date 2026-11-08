#![warn(allocation_in_loop)]

const LOOP_LIMIT: usize = 4;
const SAMPLE_ITEMS: &[u8] = &[1, 2, 3];

fn allocate_in_loop(limit: usize) {
    let mut index = 0;
    while index < limit {
        // Should warn: `with_capacity` allocates on every iteration.
        let values: Vec<u8> = Vec::with_capacity(limit);
        std::hint::black_box(values);
        // Should warn: `Box::new` allocates.
        let boxed = Box::new(index);
        std::hint::black_box(boxed);
        // Should warn: `String::from` copies into a new heap buffer.
        let label = String::from("item");
        std::hint::black_box(label);
        // Should warn: `to_string` allocates a `String`.
        let rendered = index.to_string();
        std::hint::black_box(rendered);
        // Should NOT warn: empty constructors do not allocate.
        let empty: Vec<u8> = Vec::new();
        std::hint::black_box(empty);
        index += 1;
    }
}

fn copy_in_loop(items: &[u8]) {
    for _ in items {
        // Should warn: `to_vec` allocates a `Vec`.
        let copy = items.to_vec();
        std::hint::black_box(copy);
    }
}

fn reuse_capacity(items: &[u8]) {
    // Should NOT warn: capacity is reserved before iteration and reused.
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        values.clear();
        values.push(*item);
        std::hint::black_box(&values);
    }
}

fn allocate_outside_loop() {
    // Should NOT warn: the allocation is outside a loop.
    let values: Vec<u8> = Vec::with_capacity(LOOP_LIMIT);
    std::hint::black_box(values);
}

fn main() {
    allocate_in_loop(LOOP_LIMIT);
    copy_in_loop(SAMPLE_ITEMS);
    reuse_capacity(SAMPLE_ITEMS);
    allocate_outside_loop();
}
