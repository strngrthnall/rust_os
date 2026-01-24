// =============================================================================
// Rust OS - Seguindo o tutorial Blog OS: https://os.phil-opp.com
// =============================================================================
//
// Este é um kernel mínimo escrito em Rust para aprendizado de desenvolvimento
// de sistemas operacionais. Atualmente implementa saída de texto via VGA buffer.
//
// ## Como fazer a build:
//
// Para compilar este projeto para o target x86_64 bare-metal customizado:
//
//     cargo build --target x86_64-rust_os.json
//
// Para criar uma imagem bootável:
//
//     cargo bootimage --target x86_64-rust_os.json
//
// Para executar no QEMU:
//
//     qemu-system-x86_64 -drive format=raw,file=target/x86_64-rust_os/debug/bootimage-rust_os.bin
//
// ## Pré-requisitos:
//   1. Rust nightly: rustup override set nightly
//   2. Componente rust-src: rustup component add rust-src
//   3. Componente llvm-tools: rustup component add llvm-tools-preview
//   4. Ferramenta bootimage: cargo install bootimage
//   5. QEMU para emulação: sudo apt install qemu-system-x86
//
// =============================================================================

// Desabilita a biblioteca padrão do Rust.
// Em um ambiente bare-metal não temos sistema operacional, então não podemos
// usar funcionalidades que dependem do OS (como I/O, threads, heap, etc.)
#![no_std]

// Desabilita o ponto de entrada padrão do Rust (fn main).
// O runtime do Rust normalmente chama a função main(), mas em bare-metal
// precisamos definir nosso próprio ponto de entrada.
#![no_main]

use core::panic::PanicInfo;

/// Função de tratamento de panic.
/// 
/// Em Rust, quando ocorre um panic (erro irrecuperável), o runtime normalmente
/// faz o "unwinding" da stack e mostra uma mensagem de erro. Como não temos
/// runtime em bare-metal, precisamos definir nosso próprio handler.
/// 
/// O tipo de retorno `!` (never type) indica que esta função nunca retorna.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Por enquanto, apenas entramos em um loop infinito quando há um panic.
    // Futuramente, podemos implementar logging ou reset do sistema.
    loop {}
}

/// Mensagem de boas-vindas exibida na inicialização do kernel.
/// 
/// Esta string é armazenada como um slice de bytes (`&[u8]`) para facilitar
/// a escrita direta no VGA buffer, que espera bytes ASCII.
static HELLO: &[u8] = b"Hello World!";

/// Ponto de entrada do kernel.
/// 
/// Esta função é chamada pelo bootloader quando o sistema inicia.
/// 
/// ## Atributos:
/// 
/// - `#[no_mangle]`: Impede que o compilador altere o nome da função durante
///   a compilação (name mangling). O bootloader espera encontrar `_start`.
/// 
/// - `extern "C"`: Usa a convenção de chamada C, que é o padrão para
///   interoperabilidade com código de baixo nível.
/// 
/// - O retorno `!` indica que esta função nunca retorna (diverging function),
///   pois é o ponto de entrada do sistema - não há para onde retornar!
/// 
/// ## VGA Text Buffer:
/// 
/// O VGA text buffer está mapeado na memória física no endereço `0xb8000`.
/// Cada caractere na tela é representado por 2 bytes:
/// - **Byte 0**: Código ASCII do caractere
/// - **Byte 1**: Código de cor (4 bits foreground + 4 bits background)
/// 
/// O código de cor `0x0b` significa:
/// - Foreground: `0xb` = Ciano claro (Light Cyan)
/// - Background: `0x0` = Preto
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Endereço do VGA text buffer - memória mapeada para a saída de vídeo em modo texto
    let vga_buffer = 0xb8000 as *mut u8;

    // Escreve cada caractere da mensagem HELLO no VGA buffer
    // Cada caractere ocupa 2 bytes: ASCII + atributo de cor
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            // Posição do caractere ASCII (índice * 2)
            *vga_buffer.offset(i as isize * 2) = byte;
            // Posição do atributo de cor (índice * 2 + 1)
            // 0x0b = ciano claro sobre fundo preto
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    
    // Loop infinito - o kernel deve rodar indefinidamente
    // Nos próximos capítulos do Blog OS, adicionaremos:
    // - Abstração do VGA buffer com tipos seguros
    // - Testes automatizados
    // - Tratamento de interrupções
    // - E muito mais!
    loop {}
}