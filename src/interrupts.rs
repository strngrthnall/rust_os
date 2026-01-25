//! Tratamento de interrupções e exceções da CPU.
//!
//! Configura a IDT (Interrupt Descriptor Table) com handlers para exceções.

use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// Handler para exceção de breakpoint (int3).
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

lazy_static! {
    /// IDT global com handlers de exceção configurados.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

/// Carrega a IDT no registrador IDTR.
pub fn init_idt() {
    IDT.load();
}

/// Testa se breakpoint exception é tratada corretamente.
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}