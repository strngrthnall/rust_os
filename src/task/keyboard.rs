//! # Driver Assíncrono de Teclado
//!
//! ## Como funciona?
//!
//! O teclado gera IRQs que são tratadas pelo handler em `interrupts.rs`.
//! O handler chama `add_scancode()` que coloca o scancode numa fila
//! e acorda a task que está aguardando.
//!
//! ```text
//! Tecla → IRQ1 → keyboard_handler → add_scancode()
//!                                         │
//!                                         v
//!                              SCANCODE_QUEUE.push()
//!                                         │
//!                                         v
//!                                  WAKER.wake()
//!                                         │
//!                                         v
//!                              Executor acorda task
//!                                         │
//!                                         v
//!                              ScancodeStream.poll_next()
//!                                         │
//!                                         v
//!                              print_keypresses() processa
//! ```
//!
//! ## Componentes
//!
//! - **SCANCODE_QUEUE**: Fila lock-free (ArrayQueue) de scancodes
//! - **WAKER**: AtomicWaker que guarda o waker da task consumidora
//! - **ScancodeStream**: Implementa `Stream` para consumo assíncrono
//!
//! ## Por que lock-free?
//!
//! O handler de interrupção não pode bloquear! Se usássemos Mutex,
//! poderíamos ter deadlock se a interrupção ocorresse enquanto
//! a task segura o lock.
//!
//! ## Estudo baseado em
//!
//! [Async/Await](https://os.phil-opp.com/async-await/) - Blog OS

use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

/// Fila de scancodes recebidos do teclado.
/// Inicializada lazily na primeira chamada a `ScancodeStream::new()`.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// Waker atômico que armazena o waker da task consumidora.
/// Quando um scancode chega, acordamos a task para processá-lo.
static WAKER: AtomicWaker = AtomicWaker::new();

/// Adiciona um scancode à fila. Chamado pelo handler de interrupção.
///
/// Esta função é `pub(crate)` pois só deve ser chamada por `interrupts.rs`.
/// É segura para uso em contexto de interrupção (lock-free).
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // Acorda a task que está aguardando scancodes
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

/// Stream assíncrono de scancodes do teclado.
///
/// Implementa a trait `Stream` do futures_util, permitindo
/// consumo com `while let Some(scancode) = stream.next().await`.
pub struct ScancodeStream {
    _private: (), // Previne construção externa
}

impl ScancodeStream {
    /// Cria um novo ScancodeStream, inicializando a fila global.
    ///
    /// # Panics
    /// Entra em panic se chamado mais de uma vez.
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

/// Implementação do Stream para consumo assíncrono de scancodes.
impl Stream for ScancodeStream {
    type Item = u8;

    /// Tenta obter o próximo scancode.
    ///
    /// Se a fila está vazia, registra o waker e retorna Pending.
    /// Quando `add_scancode()` chamar `wake()`, seremos acordados.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        // Fast path: se há scancode disponível, retorna imediatamente
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // Slow path: registra waker e verifica novamente
        // (evita race condition se scancode chegou entre as operações)
        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take(); // Remove waker pois temos dado
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

/// Função assíncrona que processa e imprime teclas pressionadas.
///
/// Usa pc-keyboard para decodificar scancodes em caracteres/teclas.
/// Roda indefinidamente, aguardando e processando cada tecla.
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}