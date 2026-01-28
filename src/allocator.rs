//! # Heap Allocator - Alocação Dinâmica de Memória
//!
//! ## Por que precisamos de um heap?
//!
//! Sem heap, só podemos usar a stack (tamanho fixo) e variáveis estáticas.
//! Com heap, podemos usar `Box`, `Vec`, `String`, `Rc`, etc.
//!
//! ## Como funciona?
//!
//! 1. Reservamos uma região de memória virtual para o heap (100KB em 0x4444_4444_0000)
//! 2. Mapeamos essas páginas virtuais para frames físicos
//! 3. Um allocator gerencia essa região, atendendo `alloc` e `dealloc`
//!
//! ## Implementações Disponíveis
//!
//! | Allocator | Alocação | Liberação | Fragmentação |
//! |-----------|----------|-----------|---------------|
//! | Bump | O(1) | Todas juntas | Nenhuma (linear) |
//! | Linked List | O(n) | O(1) | Alta |
//! | Fixed Size Block | O(1) | O(1) | Baixa (interna) |
//!
//! O **Fixed Size Block** é usado por padrão por ter melhor performance
//! e fragmentação controlada.
//!
//! ## Estudo baseado em
//!
//! [Heap Allocation](https://os.phil-opp.com/heap-allocation/) - Blog OS

use fixed_size_block::FixedSizeBlockAllocator;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

/// Início do heap na memória virtual.
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// Tamanho do heap (100 KB).
pub const HEAP_SIZE: usize = 100 * 1024;

// #[global_allocator]
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

// #[global_allocator]
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

/// Inicializa o heap mapeando páginas e configurando o allocator.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

/// Wrapper com spinlock para allocators.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    #[allow(mismatched_lifetime_syntaxes)]
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Alinha um endereço para cima.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}