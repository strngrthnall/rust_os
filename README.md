# Rust OS

Sistema operacional mínimo em Rust, baseado no tutorial [Writing an OS in Rust](https://os.phil-opp.com).

## Progresso

- [x] Kernel freestanding (`#![no_std]`, `#![no_main]`)
- [x] Target customizado x86_64
- [x] VGA text buffer com macros `print!` e `println!`
- [x] Handler de panic
- [ ] Testes automatizados
- [ ] Interrupções
- [ ] Gerenciamento de memória

## Quick Start

```bash
# Pré-requisitos
rustup override set nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
sudo apt install qemu-system-x86  # Ubuntu/Debian

# Executar
cargo run
```

## Comandos

| Comando | Descrição |
|---------|----------|
| `cargo build` | Compila o kernel |
| `cargo bootimage` | Cria imagem bootável |
| `cargo run` | Executa no QEMU |

## Estrutura

```
rust_os/
├── Cargo.toml
├── x86_64-rust_os.json      # Target customizado
├── .cargo/config.toml       # Configuração de build
└── src/
    ├── main.rs              # Entry point
    └── vga_buffer.rs        # Driver VGA + macros print
```

## VGA Buffer

O módulo `vga_buffer` implementa escrita no VGA text mode:

- **Endereço**: `0xb8000`
- **Dimensões**: 80x25 caracteres
- **Cores**: 16 foreground + 16 background
- **Macros**: `print!` e `println!` (similar à stdlib)

## Referências

- [Blog OS](https://os.phil-opp.com)
- [OSDev Wiki](https://wiki.osdev.org/)
