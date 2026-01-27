//! Rust OS - Entry point do kernel.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

/// Entry point chamado pelo bootloader.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    use x86_64::registers::control::Cr3;
    
    println!("Hello World{}", "!");

    rust_os::init();

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());


    #[cfg(test)]
    test_main();

    rust_os::hlt_loop();
}

/// Panic handler para modo normal (exibe no VGA).
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

/// Panic handler para testes (usa serial port).
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}