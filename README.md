# Rust OS

Sistema operacional mínimo em Rust, baseado no tutorial [Writing an OS in Rust](https://os.phil-opp.com).

## Progresso

- [x] Kernel freestanding (`#![no_std]`, `#![no_main]`)
- [x] Target customizado x86_64
- [x] VGA text buffer com macros `print!` e `println!`
- [x] Handler de panic
- [x] Testes automatizados com framework customizado
- [x] Serial port para output de testes (UART 16550)
- [x] Integração com QEMU (exit codes)
- [x] CPU Exceptions (IDT + breakpoint handler)
- [ ] Double Faults
- [ ] Interrupções de hardware (timer, teclado)
- [ ] Gerenciamento de memória

## Quick Start

```bash
# Pré-requisitos
rustup override set nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
sudo apt install qemu-system-x86  # Ubuntu/Debian
sudo pacman -S qemu-system-x86 # Arch

# Executar
cargo run

# Testes
cargo test
```

## Comandos

| Comando | Descrição |
|---------|----------|
| `cargo build` | Compila o kernel |
| `cargo run` | Executa no QEMU |
| `cargo test` | Executa todos os testes |

## Estrutura

```
rust_os/
├── Cargo.toml
├── x86_64-rust_os.json      # Target customizado
├── .cargo/config.toml       # Configuração de build
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Biblioteca + test framework
│   ├── vga_buffer.rs        # Driver VGA
│   ├── serial.rs            # Driver serial UART
│   └── interrupts.rs        # IDT e handlers de exceção
└── tests/
    ├── basic_boot.rs
    └── should_panic.rs
```

## Arquitetura

### Módulos

| Módulo | Descrição |
|--------|----------|
| `vga_buffer` | VGA text mode (0xb8000), 80x25 |
| `serial` | UART 16550 (0x3F8) para testes |
| `interrupts` | IDT com handler de breakpoint |

## Referências

- [Blog OS](https://os.phil-opp.com)
- [OSDev Wiki](https://wiki.osdev.org/)
