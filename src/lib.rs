#![no_std]

mod modules;
mod nanolibc;

pub mod allocation;
pub mod dll;
pub mod sys;
pub mod fs;


pub use modules::*;

#[macro_use]
extern crate alloc;

#[macro_export]
macro_rules! throw_new {
    ($text:expr) => {
        debug_write(&format!(
            "{}:{}\nAn error raised:\n{}\n",
            file!(),
            line!(),
            $text
        ));
    };
}

#[macro_export]
macro_rules! panic {
    ($text:expr) => {
        debug_write(cstr_core::cstr!(concat!("Panic!\n", $text, "\n")));
        unsafe {
            sys::exit();
        }
    };
}

