//! # Gerenciamento de Memória: Paginação e Frame Allocation
//!
//! ## Paginação no x86_64
//!
//! O x86_64 usa paginação de 4 níveis para traduzir endereços virtuais em físicos:
//!
//! ```text
//! Endereço Virtual (48 bits usados)
//! ┌─────────┬─────────┬─────────┬─────────┬────────────┐
//! │ P4 Index│ P3 Index│ P2 Index│ P1 Index│ Page Offset  │
//! │ (9 bits)│ (9 bits)│ (9 bits)│ (9 bits)│  (12 bits)   │
//! └────┬────┴────┬────┴────┬────┴────┬────┴──────┬─────┘
//!      │          │          │          │              │
//!      v          v          v          v              v
//!    P4 Table → P3 Table → P2 Table → P1 Table → Frame + Offset
//!    (CR3)      (512 entries cada, 8 bytes/entry)
//! ```
//!
//! ## Frame Allocator
//!
//! O bootloader fornece um memory map indicando regiões usáveis.
//! O `BootInfoFrameAllocator` itera sobre essas regiões para alocar
//! frames físicos de 4KB sob demanda.
//!
//! ## Offset Mapping
//!
//! O bootloader mapeia TODA a memória física em um offset virtual.
//! Ex: Se offset = 0x1000_0000_0000, então:
//!   - Memória física 0x0 está em virtual 0x1000_0000_0000
//!   - Memória física 0x1000 está em virtual 0x1000_0000_1000
//!
//! Isso permite acessar qualquer endereço físico facilmente.
//!
//! ## Estudo baseado em
//!
//! [Introduction to Paging](https://os.phil-opp.com/paging-introduction/) - Blog OS

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PageTableFlags as Flags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Cria um mapeamento de exemplo para o VGA buffer.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

/// Frame allocator vazio (não aloca nada).
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// Frame allocator que usa o memory map do bootloader.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Cria um allocator a partir do memory map do bootloader.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Retorna um iterador sobre os frames usáveis.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Inicializa o OffsetPageTable a partir do offset de memória física.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

/// Retorna uma referência mutável para a page table de nível 4 ativa.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

/// Traduz um endereço virtual para físico.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_mut_ptr();
        let table = unsafe { &*table_ptr };
        let entry = &table[index];

        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        }
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}