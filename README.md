# Rust OS

Um sistema operacional simples escrito em Rust, seguindo o tutorial [Writing an OS in Rust](https://os.phil-opp.com) do Philipp Oppermann.

## 📚 Sobre o Projeto

Este projeto é um estudo de desenvolvimento de sistemas operacionais utilizando a linguagem Rust. O objetivo é aprender conceitos fundamentais de sistemas operacionais como:

- Programação bare-metal (sem sistema operacional subjacente)
- Gerenciamento de memória
- Interrupções e exceções
- Drivers de dispositivos
- E muito mais!

## 🛠️ Pré-requisitos

- [Rust](https://www.rust-lang.org/tools/install) (versão nightly recomendada)
- Componente `rust-src` (para compilação cross-platform)

### Instalação dos pré-requisitos

```bash
# Instalar Rust (se ainda não tiver)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Usar a versão nightly
rustup override set nightly

# Instalar o componente rust-src (necessário para compilar para targets bare-metal)
rustup component add rust-src

# Instalar o target para ARM Cortex-M (se necessário)
rustup target add thumbv7em-none-eabihf
```

## 🔨 Build

Para compilar o projeto para um target bare-metal (sem sistema operacional), use:

```bash
cargo build --target thumbv7em-none-eabihf
```

### Explicação do comando de build:

- `cargo build`: Comando padrão do Cargo para compilar o projeto
- `--target thumbv7em-none-eabihf`: Especifica o target de compilação
  - `thumbv7em`: Arquitetura ARM Cortex-M4/M7 com instruções Thumb
  - `none`: Sem sistema operacional (bare-metal)
  - `eabihf`: Embedded ABI com hardware floating-point

### Outros targets úteis:

```bash
# Para x86_64 (usado no tutorial blog_os)
cargo build --target x86_64-unknown-none

# Build em modo release (otimizado)
cargo build --target thumbv7em-none-eabihf --release
```

## 📁 Estrutura do Projeto

```
rust_os/
├── Cargo.toml      # Configuração do projeto e dependências
├── src/
│   └── main.rs     # Ponto de entrada do kernel
└── target/         # Arquivos compilados
```

## 📖 Referências

- [Writing an OS in Rust (Blog OS)](https://os.phil-opp.com) - Tutorial principal
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [Rust OSDev](https://rust-osdev.com/)

## 📝 Licença

Este projeto é apenas para fins educacionais.
