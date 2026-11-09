use std::sync::mpsc::channel as std_channel;

use crossbeam::channel::unbounded as crossbeam_unbounded;
use tokio::sync::mpsc::unbounded_channel as tokio_unbounded_channel;

fn std_unbounded() {
    let _ = std_channel::<u8>();
}

fn crossbeam_unbounded_ctor() {
    let _ = crossbeam_unbounded::<u8>();
}

fn tokio_unbounded_ctor() {
    let _ = tokio_unbounded_channel::<u8>();
}

fn tokio_bounded_is_allowed() {
    let _ = tokio::sync::mpsc::channel::<u8>(32);
}

fn std_unbounded_unit_turbofish() {
    // Only the argument list gains the capacity placeholder, not `::<()>`.
    let _ = std::sync::mpsc::channel::<()>();
}

fn main() {
    std_unbounded();
    crossbeam_unbounded_ctor();
    tokio_unbounded_ctor();
    tokio_bounded_is_allowed();
    std_unbounded_unit_turbofish();
}

use crossbeam::channel::unbounded;
use tokio::sync::mpsc as tokio_mpsc;

fn crossbeam_unaliased_ctor() {
    // The callee keeps its original name, so only that segment is renamed.
    let _ = unbounded::<u8>();
}

fn tokio_module_alias_ctor() {
    // A module alias keeps the original function segment, so only it is renamed.
    let _ = tokio_mpsc::unbounded_channel::<u8>();
}

mod queues {
    pub use std::sync::mpsc::channel;
    pub use std::sync::mpsc::channel as make_queue;
}

use queues::channel as local_queue;

fn module_reexported_alias_ctor() {
    // The alias import starts at the `std` crate root, so its module path is reused.
    let _ = queues::make_queue::<u8>();
}

fn local_module_alias_ctor() {
    // The alias import starts at a local module, so no rewrite is offered.
    let _ = local_queue::<u8>();
}
