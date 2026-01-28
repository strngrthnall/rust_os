//! # GDT (Global Descriptor Table) e TSS (Task State Segment)
//!
//! ## Por que precisamos da GDT?
//!
//! Em modo protegido/long mode, a GDT define segmentos de memória.
//! No x86_64, a segmentação é praticamente ignorada (flat memory),
//! mas ainda precisamos de entries para:
//!
//! - **Kernel Code Segment**: Define o nível de privilégio (ring 0)
//! - **TSS Segment**: Aponta para a Task State Segment
//!
//! ## Por que precisamos da TSS?
//!
//! A TSS contém a **Interrupt Stack Table (IST)**, que permite
//! usar stacks diferentes para handlers de interrupção.
//!
//! Isso é **crítico** para double faults: se a stack estourou,
//! o handler precisa de uma stack limpa para executar.
//! Sem isso, teríamos um **triple fault** (reset da CPU).
//!
//! ## Fluxo
//!
//! ```text
//! init() → Carrega GDT → Atualiza CS (code segment)
//!                      → Carrega TSS no registrador TR
//! ```
//!
//! ## Estudo baseado em
//!
//! [Double Faults](https://os.phil-opp.com/double-fault-exceptions/) - Blog OS

use lazy_static::lazy_static;
use x86_64::{
    instructions::{
        segmentation::{Segment, CS},
        tables::load_tss,
    },
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};


/// Índice na IST para a stack de double fault.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Seletores de segmento para code e TSS.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    /// TSS com stack dedicada para double faults (20KB).
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(&raw const STACK) + STACK_SIZE
        };
        tss
    };

    /// GDT com segmentos de kernel code e TSS.
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

/// Carrega a GDT e configura os registradores CS e TSS.
pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}