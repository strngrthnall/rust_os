//! Rust OS - Kernel mínimo seguindo o Blog OS (https://os.phil-opp.com)
//!
//! Executar: `cargo run`

#![no_std]  // Sem biblioteca padrão (bare-metal)
#![no_main] // Entry point customizado (_start)

mod vga_buffer;

use core::panic::PanicInfo;

/// Handler de panic - exibe a mensagem e entra em loop infinito.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// Entry point do kernel, chamado pelo bootloader.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");
    panic!("Some panic message");

    #[allow(unreachable_code)]
    loop {}
}