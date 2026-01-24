// =============================================================================
// Rust OS - Seguindo o tutorial Blog OS: https://os.phil-opp.com
// =============================================================================
//
// Este é um kernel mínimo escrito em Rust para aprendizado de desenvolvimento
// de sistemas operacionais.
//
// ## Como fazer a build:
//
// Para compilar este projeto para um target bare-metal, execute:
//
//     cargo build --target thumbv7em-none-eabihf
//
// Onde:
//   - thumbv7em: Arquitetura ARM Cortex-M4/M7 (instruções Thumb)
//   - none: Sem sistema operacional (bare-metal/freestanding)
//   - eabihf: Embedded ABI com suporte a hardware floating-point
//
// Pré-requisitos:
//   1. Instalar o target: rustup target add thumbv7em-none-eabihf
//   2. Ter o componente rust-src: rustup component add rust-src
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

/// Ponto de entrada do kernel.
/// 
/// Esta função é chamada pelo bootloader quando o sistema inicia.
/// 
/// - `#[no_mangle]`: Impede que o compilador altere o nome da função durante
///   a compilação (name mangling). O bootloader espera encontrar `_start`.
/// 
/// - `extern "C"`: Usa a convenção de chamada C, que é o padrão para
///   interoperabilidade com código de baixo nível.
/// 
/// - O retorno `!` indica que esta função nunca retorna (diverging function),
///   pois é o ponto de entrada do sistema - não há para onde retornar!
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Loop infinito - por enquanto nosso kernel não faz nada.
    // Nos próximos capítulos do Blog OS, adicionaremos:
    // - Saída de texto (VGA buffer)
    // - Testes
    // - Tratamento de exceções
    // - E muito mais!
    loop {}
}