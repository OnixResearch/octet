mod guest {
    pub fn spawn_guest_vm() {}
}

mod storage {
    // Should warn: identifiers in the item body are not documentation.
    pub fn storage_flush() -> u32 {
        let rapid = 1;
        rapid
    }
}

mod path_segment_repetition {
    // Should NOT warn: the repeated identity ties lint source, fixture, and metadata together.
    pub fn path_segment_repetition_classifier() {}
}

fn main() {
    guest::spawn_guest_vm();
    let _ = storage::storage_flush();
    path_segment_repetition::path_segment_repetition_classifier();
}
