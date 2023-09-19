#![deny(missing_docs)]
//! An example of using subenum with `#![deny(missing_docs)]`.
//!
//! This is an example because the aforementioned attribute doesn't work in
//! tests.

use subenum::subenum;

/// An enum.
#[subenum(Bar)]
pub enum Foo {
    /// Variant A
    #[subenum(Bar)]
    A,
    /// Variant B
    B,
}

fn main() {}
