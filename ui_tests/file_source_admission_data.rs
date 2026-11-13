#![deny(excessive_file_length, module_file_count, underscore_in_module_filename)]

// These files are payloads only, including Rust-looking data under an ordinary
// vendor path. Neither their names, lengths, nor directory count are source findings.
const DATA: [&str; 11] = [
    include_str!("../policy_inputs/underscore_module_filename/vendor/mod.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/alpha.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/beta.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/gamma.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/delta.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/epsilon.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/zeta.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/eta.rs"),
    include_str!("../policy_inputs/underscore_module_filename/vendor/compute_vm.rs"),
    include_str!("../policy_inputs/source_admission/records.json"),
    include_str!("../policy_inputs/source_admission/NOTICE"),
];
const BYTES: [&[u8]; 2] = [
    include_bytes!("../policy_inputs/source_admission/records.json"),
    include_bytes!("../policy_inputs/underscore_module_filename/vendor/compute_vm.rs"),
];

fn main() {}
