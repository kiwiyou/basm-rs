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

#[cfg_attr(test, allow(dead_code))]
#[path = "../solution.rs"]
mod solution;
mod codegen;

#[panic_handler]
fn panic(_pi: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}