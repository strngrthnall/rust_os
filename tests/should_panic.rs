//! Teste de integração: verifica que panic funciona corretamente.
//!
//! Este teste DEVE panic - sucesso é quando o panic handler é chamado.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rust_os::{exit_qemu, QemuExitCode, serial_print, serial_println};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Função que deve falhar (assert_eq!(0, 1)).
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

/// Panic handler customizado - retorna sucesso pois o teste DEVE panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}