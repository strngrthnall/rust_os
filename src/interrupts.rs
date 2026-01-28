//! # Tratamento de Interrupções e Exceções
//!
//! ## O que são interrupções?
//!
//! Interrupções são sinais que pausam a execução normal da CPU
//! para executar um handler específico. Existem dois tipos:
//!
//! - **Exceções**: Geradas pela CPU (division by zero, page fault, etc.)
//! - **IRQs**: Geradas por hardware externo (timer, teclado, etc.)
//!
//! ## IDT (Interrupt Descriptor Table)
//!
//! Tabela com 256 entries que mapeia números de interrupção para handlers:
//!
//! - **0-31**: Exceções reservadas pela CPU
//! - **32-47**: IRQs remapeadas (originalmente 0-15)
//! - **48-255**: Livres para uso
//!
//! ## PIC 8259 (Programmable Interrupt Controller)
//!
//! O PIC converte IRQs de hardware em interrupções para a CPU.
//! São 2 PICs encadeados (master + slave) = 16 IRQs.
//!
//! Por padrão, IRQs 0-7 mapeiam para interrupções 0-7, que colidem
//! com exceções! Por isso remapeamos para 32-47.
//!
//! ## Fluxo de uma interrupção
//!
//! ```text
//! Hardware/CPU → Interrupção N → IDT[N] → Handler → EOI → Retorna
//! ```
//!
//! ## Estudo baseado em
//!
//! - [CPU Exceptions](https://os.phil-opp.com/cpu-exceptions/)
//! - [Hardware Interrupts](https://os.phil-opp.com/hardware-interrupts/)

use crate::{gdt, hlt_loop, print, println};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{
    instructions::port::Port,
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};


/// Índices das interrupções de hardware (IRQs remapeadas).
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl From<InterruptIndex> for u8 {
    fn from(ii: InterruptIndex) -> Self {
        ii as u8
    }
}

impl From<InterruptIndex> for usize {
    fn from(ii: InterruptIndex) -> Self {
        ii as usize
    }
}


extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}


// ============================================================================
// PICs 8259
// ============================================================================

/// Offset do PIC 1 (IRQ 0-7 → interrupções 32-39).
pub const PIC_1_OFFSET: u8 = 32;
/// Offset do PIC 2 (IRQ 8-15 → interrupções 40-47).
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// PICs encadeados (master + slave) com mutex para acesso thread-safe.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(
        unsafe { 
            ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) 
        }
    );

// ============================================================================
// Exception Handlers
// ============================================================================

/// Handler para exceção de breakpoint (int3).
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler para double fault - usa stack separada (IST) para evitar triple fault.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// ============================================================================
// Hardware Interrupt Handlers
// ============================================================================

/// Handler do timer (IRQ 0) - imprime um ponto a cada tick.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.into());
    }
}

/// Handler do teclado (IRQ 1) - lê scancode e imprime caractere.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.into());
    }
}

// ============================================================================
// IDT
// ============================================================================

lazy_static! {
    /// IDT global com handlers de exceção e interrupção configurados.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.into()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.into()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
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