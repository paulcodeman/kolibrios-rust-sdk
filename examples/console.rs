#![no_std]
#![no_main]

use kos::{dll::Console, threads::exit};

extern crate alloc;

#[no_mangle]
pub fn kol_main() {
    let con_lib = Console::import(None).unwrap();
    con_lib.init(u32::MAX, u32::MAX, u32::MAX, u32::MAX, c"Rust!");
    con_lib.write_string("Hi from Rust!");
    con_lib.exit(false);

    exit();
}
