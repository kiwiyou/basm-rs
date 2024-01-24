#![feature(fn_align)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![cfg_attr(not(test), no_builtins)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

extern crate basm_std as basm;

#[allow(dead_code)]
#[path = "../solution.rs"]
mod solution;

#[cfg(test)]
mod verify_test_works {
    fn add(x: i64, y: i64) -> i64 {
        x + y
    }

    #[test]
    fn run() {
        assert_eq!(8, add(5, 3));
    }
}