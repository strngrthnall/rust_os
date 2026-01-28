//! # Driver Serial UART 16550
//!
//! ## O que é UART?
//!
//! Universal Asynchronous Receiver/Transmitter - um chip que converte
//! dados paralelos em serial e vice-versa. O 16550 é uma versão comum
//! que suporta FIFOs para bufferização.
//!
//! ## Por que serial é útil?
//!
//! - **Testes**: QEMU redireciona serial para stdout do host
//! - **Debug**: Funciona mesmo quando VGA não está disponível
//! - **Simplicidade**: Não requer driver de vídeo complexo
//!
//! ## Portas Serial no PC
//!
//! | Porta | Endereço I/O | IRQ |
//! |-------|--------------|-----|
//! | COM1  | 0x3F8        | 4   |
//! | COM2  | 0x2F8        | 3   |
//! | COM3  | 0x3E8        | 4   |
//! | COM4  | 0x2E8        | 3   |
//!
//! ## Integração com Test Framework
//!
//! Os testes usam `serial_println!` para reportar resultados.
//! QEMU é configurado com `-serial stdio` para exibir a saída.
//!
//! ## Estudo baseado em
//!
//! [Testing](https://os.phil-opp.com/testing/) - Blog OS

use core::fmt;
use fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

// Porta serial COM1 (0x3F8) com mutex para acesso thread-safe.
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
    })
}

/// Macro para print na serial sem newline.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Macro para print na serial com newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}