//! # Rust OS - Entry Point do Kernel
//!
//! Este é o ponto de entrada do kernel após o bootloader carregar o sistema.
//!
//! ## Sequência de Inicialização
//!
//! 1. Bootloader carrega o kernel e salta para `_start`
//! 2. `kernel_main` recebe informações do boot (memory map, etc.)
//! 3. `rust_os::init()` configura GDT, IDT e PICs
//! 4. Configura paginação e frame allocator
//! 5. Inicializa o heap para alocação dinâmica
//! 6. Cria o executor e spawna tasks assíncronas
//! 7. Entra no loop do executor (nunca retorna)
//!
//! ## Estudo baseado em
//!
//! [Writing an OS in Rust](https://os.phil-opp.com) - Philipp Oppermann

#![no_std]  // Não usa biblioteca padrão (depende de OS)
#![no_main] // Entry point customizado, não usa `fn main()`
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;  // Habilita tipos de alocação (Box, Vec, etc.)

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::{
    allocator,
    memory::{self, BootInfoFrameAllocator},
    println,
    task::{executor::Executor, keyboard, Task},
};
use x86_64::VirtAddr;

// Macro que gera o entry point `_start` com assinatura correta
entry_point!(kernel_main);

/// Entry point chamado pelo bootloader.
#[unsafe(no_mangle)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    
    println!("Hello World{}", "!");

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    println!("Physical Memory Offset ... [ok]");
    let mut mapper = unsafe {
        memory::init(phys_mem_offset)
    };
    println!("Memory Mapper initiated ... [ok]");
    
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    println!("Frame Allocator initiated ... [ok]");

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("Heap Memory initiated ... [ok]");

    // let heap_value = Box::new(41);
    // println!("heap_value at {:p}", heap_value);

    // let mut vec = Vec::new();
    // for i in 0..500 {
    //     vec.push(i)
    // }
    // println!("vec at {:p}", vec.as_slice());

    // let reference_counted = Rc::new(vec![1,2,3]);
    // let cloned_reference = reference_counted.clone();
    // println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    // core::mem::drop(reference_counted);
    // println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    let mut executor = Executor::new();
    println!("Simple Executor created ... [ok]");
    executor.spawn(Task::new(example_task()));
    println!("Example Task spawned ... [ok]");
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
    println!("Tasks running ... [ok]");

    #[cfg(test)]
    test_main();
    println!("It did not crash!");

    rust_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async unmber: {}", number)
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