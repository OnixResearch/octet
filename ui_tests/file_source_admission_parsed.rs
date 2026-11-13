// A data load must not prevent the same file from being admitted when parsed.
const DATA: &str = include_str!("../policy_inputs/underscore_module_filename/vendor/compute_vm.rs");

#[path = "../policy_inputs/underscore_module_filename/vendor/mod.rs"]
mod vendor;

// Retain parsed include! coverage as well as out-of-line module coverage.
include!("../policy_inputs/registry/oversized_registry_module.rs");

fn main() {}
