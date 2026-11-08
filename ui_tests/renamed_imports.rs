#![allow(dead_code, unused_imports)]

mod transport {
    pub struct Config;
}

mod rustc_hir {
    pub struct Item;
}

// Should NOT warn: canonical rustc-private shorthand is scoped to lint implementation code.
use crate::rustc_hir as hir;

use crate::transport::Config as TransportConfig;

// Should warn: the alias spelling alone does not document a downstream contract.
pub use crate::transport::Config as IncompatibleTransportConfig;

fn connect(config: TransportConfig) {
    let _ = config;
}

fn use_rustc_alias(_: hir::Item) {}

fn main() {}
