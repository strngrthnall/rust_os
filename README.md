# Rust OS

Um sistema operacional simples escrito em Rust, seguindo o tutorial [Writing an OS in Rust](https://os.phil-opp.com) do Philipp Oppermann.

## 📚 Sobre o Projeto

Este projeto é um estudo de desenvolvimento de sistemas operacionais utilizando a linguagem Rust. O objetivo é aprender conceitos fundamentais de sistemas operacionais como:

- Programação bare-metal (sem sistema operacional subjacente)
- Saída de texto via VGA buffer
- Gerenciamento de memória
- Interrupções e exceções
- Drivers de dispositivos
- E muito mais!

## ✅ Funcionalidades Implementadas

- [x] Kernel mínimo freestanding (sem biblioteca padrão)
- [x] Target customizado x86_64 para bare-metal
- [x] Ponto de entrada `_start` com convenção C
- [x] Handler de panic customizado
- [x] **Saída de texto via VGA buffer** - Exibe "Hello World!" na tela
- [ ] Testes automatizados
- [ ] Tratamento de interrupções
- [ ] Gerenciamento de memória

## 🛠️ Pré-requisitos

- [Rust](https://www.rust-lang.org/tools/install) (versão nightly)
- Componente `rust-src` (para compilação cross-platform)
- Componente `llvm-tools-preview` (para criar imagem bootável)
- `bootimage` (ferramenta para criar imagens de boot)
- QEMU (para emulação e testes)

### Instalação dos pré-requisitos

```bash
# Instalar Rust (se ainda não tiver)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Usar a versão nightly
rustup override set nightly

# Instalar o componente rust-src (necessário para compilar para targets bare-metal)
rustup component add rust-src

# Instalar llvm-tools-preview (necessário para bootimage)
rustup component add llvm-tools-preview

# Instalar a ferramenta bootimage
cargo install bootimage

# Instalar QEMU para emulação (Ubuntu/Debian)
sudo apt install qemu-system-x86
```

## 🔨 Build

Este projeto usa um target customizado (`x86_64-rust_os.json`) que define a configuração específica para nosso kernel. Este está definido no arquivo `.cargo/config.toml`.

### Compilar o kernel:

```bash
cargo build
```

### Criar imagem bootável:

```bash
cargo bootimage
```

Isso criará uma imagem bootável em `target/x86_64-rust_os/debug/bootimage-rust_os.bin`.

### Executar no cargo:

```bash
cargo run
```

### Executar no QEMU:

```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-rust_os/debug/bootimage-rust_os.bin
```

## 🖥️ VGA Buffer

O kernel atualmente implementa saída de texto usando o **VGA text buffer**:

- **Endereço de memória**: `0xb8000`
- **Formato**: Cada caractere usa 2 bytes:
  - Byte 1: Código ASCII do caractere
  - Byte 2: Código de cor (foreground/background)
- **Cor atual**: `0x0b` (ciano claro sobre fundo preto)
- **Mensagem exibida**: "Hello World!"

## 📁 Estrutura do Projeto

```
rust_os/
├── Cargo.toml              # Configuração do projeto e dependências
├── x86_64-rust_os.json     # Target customizado para x86_64 bare-metal
├── src/
│   └── main.rs             # Ponto de entrada do kernel com VGA output
└── target/                 # Arquivos compilados
    └── x86_64-rust_os/
        └── debug/
            └── bootimage-rust_os.bin  # Imagem bootável
```

## 🧩 Arquitetura

### Target Customizado (`x86_64-rust_os.json`)

O projeto usa um target JSON customizado com as seguintes configurações:

| Configuração | Valor | Descrição |
|--------------|-------|-----------|
| `llvm-target` | `x86_64-unknown-none` | Target LLVM base |
| `arch` | `x86_64` | Arquitetura de 64 bits |
| `panic-strategy` | `abort` | Não faz unwinding em panic |
| `disable-redzone` | `true` | Desabilita a red zone (necessário para handlers de interrupção) |
| `features` | `-mmx,-sse,+soft-float` | Desabilita SIMD, usa software float |

### Componentes do Kernel (`main.rs`)

- **`#![no_std]`**: Desabilita a biblioteca padrão
- **`#![no_main]`**: Desabilita o ponto de entrada padrão
- **`panic_handler`**: Handler customizado para panics
- **`HELLO`**: String estática com a mensagem de boas-vindas
- **`_start`**: Ponto de entrada que escreve no VGA buffer

## 📖 Referências

- [Writing an OS in Rust (Blog OS)](https://os.phil-opp.com) - Tutorial principal
- [VGA Text Mode](https://os.phil-opp.com/vga-text-mode/) - Capítulo sobre VGA buffer
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [OSDev Wiki - VGA Hardware](https://wiki.osdev.org/VGA_Hardware)
- [Rust OSDev](https://rust-osdev.com/)

## 📝 Licença

Este projeto é apenas para fins educacionais.
