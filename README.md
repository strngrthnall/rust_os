# Rust OS

> **📚 Este projeto é um estudo do código [blog_os](https://github.com/phil-opp/blog_os) de Philipp Oppermann.**
>
> O objetivo é aprender desenvolvimento de sistemas operacionais seguindo o excelente tutorial [Writing an OS in Rust](https://os.phil-opp.com). Todo o código foi escrito manualmente acompanhando os posts do blog, com comentários em português para facilitar o entendimento.

## Sobre o Projeto

Este é um kernel mínimo escrito em Rust para a arquitetura x86_64. O projeto demonstra os conceitos fundamentais de um sistema operacional, desde a inicialização bare-metal até multitasking cooperativo com async/await.

## Progresso do Tutorial

### ✅ Etapas Concluídas

| Etapa | Descrição | Post Original |
|-------|-----------|---------------|
| 1. Freestanding Binary | Kernel sem stdlib, `#![no_std]` e `#![no_main]` | [A Freestanding Rust Binary](https://os.phil-opp.com/freestanding-rust-binary/) |
| 2. Minimal Kernel | Target x86_64 customizado, bootloader, entry point `_start` | [A Minimal Rust Kernel](https://os.phil-opp.com/minimal-rust-kernel/) |
| 3. VGA Text Mode | Driver para buffer VGA em 0xb8000, macros `print!`/`println!` | [VGA Text Mode](https://os.phil-opp.com/vga-text-mode/) |
| 4. Testing | Framework de testes customizado, saída via serial port | [Testing](https://os.phil-opp.com/testing/) |
| 5. CPU Exceptions | IDT (Interrupt Descriptor Table), handler de breakpoint | [CPU Exceptions](https://os.phil-opp.com/cpu-exceptions/) |
| 6. Double Faults | GDT, TSS, IST para tratar double faults com stack separada | [Double Faults](https://os.phil-opp.com/double-fault-exceptions/) |
| 7. Hardware Interrupts | PIC 8259, handlers de timer e teclado | [Hardware Interrupts](https://os.phil-opp.com/hardware-interrupts/) |
| 8. Paging | Page tables de 4 níveis, tradução de endereços, mapeamento | [Introduction to Paging](https://os.phil-opp.com/paging-introduction/) |
| 9. Heap Allocation | Frame allocator, heap mapping, allocators (bump, linked list, fixed block) | [Heap Allocation](https://os.phil-opp.com/heap-allocation/) |
| 10. Async/Await | Tasks, executors, teclado assíncrono com wakers | [Async/Await](https://os.phil-opp.com/async-await/) |

## Arquitetura do Kernel

### Inicialização (Boot)

```
BIOS/UEFI → Bootloader → _start() → kernel_main()
                              ↓
                         rust_os::init()
                              ↓
                    ┌─────────┴─────────┐
                    ↓                   ↓
               gdt::init()      interrupts::init_idt()
                    ↓                   ↓
              Carrega GDT         Carrega IDT
              Configura TSS       Inicializa PICs
                                  Habilita interrupções
```

### Gerenciamento de Memória

```
Memória Física                    Memória Virtual
┌────────────────┐               ┌────────────────┐
│   Bootloader   │               │     Kernel     │
├────────────────┤               ├────────────────┤
│     Kernel     │  ←─mapping─→  │      Heap      │ 0x4444_4444_0000
├────────────────┤               │    (100 KB)    │
│  Frames Livres │               ├────────────────┤
│   (usable)     │               │   VGA Buffer   │ 0xb8000
└────────────────┘               └────────────────┘

Frame Allocator: Aloca frames físicos de 4KB
Page Mapper: Mapeia páginas virtuais → frames físicos
Heap Allocator: Gerencia alocações dinâmicas (Box, Vec, etc.)
```

### Sistema de Interrupções

```
┌─────────────────────────────────────────────────────┐
│                        IDT                          │
├─────────────────────────────────────────────────────┤
│  0-31: Exceções da CPU (breakpoint, page fault...)  │
│ 32-39: IRQ 0-7 do PIC 1 (timer, teclado...)         │
│ 40-47: IRQ 8-15 do PIC 2                            │
└─────────────────────────────────────────────────────┘

Interrupção → Handler → EOI (End of Interrupt) → Retorna
```

### Async/Await e Multitasking

```
┌─────────────────┐     ┌─────────────────┐
│    Executor     │     │   Task Queue    │
│                 │────→│  (ArrayQueue)   │
│  run_ready()    │     └─────────────────┘
│  sleep_if_idle()│              ↑
└─────────────────┘              │
         ↓                       │ wake()
┌─────────────────┐     ┌─────────────────┐
│      Task       │     │     Waker       │
│   (Future)      │←────│  (TaskWaker)    │
└─────────────────┘     └─────────────────┘
```

## Estrutura do Código

```
src/
├── main.rs              # Entry point: inicialização e loop principal
├── lib.rs               # Biblioteca: init(), test framework, exports
│
├── vga_buffer.rs        # Driver VGA text mode (80x25, 16 cores)
├── serial.rs            # Driver UART 16550 para debug/testes
│
├── gdt.rs               # Global Descriptor Table + Task State Segment
├── interrupts.rs        # IDT + handlers (exceções e IRQs)
│
├── memory.rs            # Paginação: page tables, frame allocator
├── allocator.rs         # Heap: init_heap, Locked wrapper
├── allocator/
│   ├── bump.rs          # Bump allocator (simples, sem free individual)
│   ├── linked_list.rs   # Linked list allocator (free list)
│   └── fixed_size_block.rs  # Fixed size block (usado por padrão)
│
└── task/
    ├── mod.rs           # Task e TaskId
    ├── simple_executor.rs   # Executor básico (busy-loop)
    ├── executor.rs      # Executor otimizado (wakers, sleep)
    └── keyboard.rs      # Stream assíncrono de teclas
```

## Quick Start

```bash
# Pré-requisitos
rustup override set nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
sudo apt install qemu-system-x86  # Ubuntu/Debian

# Executar
cargo run

# Testes
cargo test
```

## Conceitos Implementados

### 1. Freestanding Binary
- `#![no_std]`: Sem biblioteca padrão (depende do OS)
- `#![no_main]`: Entry point customizado `_start`
- `#[panic_handler]`: Handler de panic próprio

### 2. VGA Text Buffer
- Memória mapeada em `0xb8000`
- 80 colunas × 25 linhas
- Cada caractere: 1 byte ASCII + 1 byte cor
- Volatile writes para evitar otimizações do compilador

### 3. Interrupções
- **IDT**: Tabela com 256 entries para handlers de interrupção
- **PIC 8259**: Controlador de interrupções de hardware (remapeado para 32-47)
- **IST**: Interrupt Stack Table - stack separada para double faults

### 4. Paginação
- Page tables de 4 níveis (P4 → P3 → P2 → P1 → Frame)
- Páginas de 4KB cada
- Identity mapping pelo bootloader
- Offset mapping para acesso à memória física

### 5. Heap Allocation
- **Bump**: Aloca sequencialmente, libera tudo junto
- **Linked List**: Free list com coalescing de regiões adjacentes
- **Fixed Size Block**: Listas separadas por tamanho (8-2048 bytes) - mais eficiente

### 6. Async/Await
- **Task**: Wrapper de Future pinned em Box
- **Executor**: Poll de tasks prontas, HLT quando ocioso
- **Waker**: Notifica executor quando I/O está disponível

## Referências

- 📖 [Writing an OS in Rust](https://os.phil-opp.com) - Tutorial original de Philipp Oppermann
- 💻 [blog_os no GitHub](https://github.com/phil-opp/blog_os) - Código fonte do tutorial
- 📚 [OSDev Wiki](https://wiki.osdev.org/) - Referência técnica de OS development
- 🦀 [The Rust Programming Language](https://doc.rust-lang.org/book/)

## Licença

Este projeto é apenas para fins educacionais, seguindo o tutorial de Philipp Oppermann.
