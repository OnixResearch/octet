//! Compile-time concatenation retains the original stable public slice and order.
mod effects;
mod interfaces;
mod safety;
mod style;

pub(super) const COLLECTED: [crate::LintMetadata; 74] = collect();

const fn collect() -> [crate::LintMetadata; 74] {
    let chunks = [style::ENTRIES, interfaces::ENTRIES, effects::ENTRIES, safety::ENTRIES];
    let mut entries = [style::ENTRIES[0]; 74];
    let mut index = 0;
    let mut chunk_index = 0;
    while chunk_index < chunks.len() {
        let chunk = chunks[chunk_index];
        let mut item_index = 0;
        while item_index < chunk.len() {
            entries[index] = chunk[item_index];
            index += 1;
            item_index += 1;
        }
        chunk_index += 1;
    }
    assert!(index == entries.len());
    entries
}
