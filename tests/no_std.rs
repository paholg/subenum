#![cfg(not(feature = "std"))]
#![no_std]

use subenum::subenum;

#[allow(unused)]
#[subenum(OnlyA)]
enum Simple {
    #[subenum(OnlyA)]
    A,
    B,
}
