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
- [ ] Interrupções
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

# Executar testes
cargo test
```

## Comandos

| Comando | Descrição |
|---------|----------|
| `cargo build` | Compila o kernel |
| `cargo run` | Executa no QEMU |
| `cargo test` | Executa todos os testes |
| `cargo test --test basic_boot` | Teste de boot básico |
| `cargo test --test should_panic` | Teste de panic |

## Estrutura

```
rust_os/
├── Cargo.toml
├── x86_64-rust_os.json      # Target customizado
├── .cargo/config.toml       # Configuração de build
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Biblioteca com test framework
│   ├── vga_buffer.rs        # Driver VGA + macros print
│   └── serial.rs            # Driver serial UART + macros
└── tests/
    ├── basic_boot.rs        # Teste de boot e println
    └── should_panic.rs      # Teste que deve panic
```

## Arquitetura

### Módulos

| Módulo | Descrição |
|--------|----------|
| `vga_buffer` | Escrita no VGA text mode (0xb8000), 80x25, 16 cores |
| `serial` | UART 16550 para output de testes via porta 0x3F8 |
| `lib` | Test runner, trait `Testable`, integração QEMU |

### Testes

O framework de testes customizado usa serial port para output e exit codes do QEMU:
- `QemuExitCode::Success` (0x10) → exit 0
- `QemuExitCode::Failed` (0x11) → exit 1

## Referências

- [Blog OS](https://os.phil-opp.com)
- [OSDev Wiki](https://wiki.osdev.org/)
