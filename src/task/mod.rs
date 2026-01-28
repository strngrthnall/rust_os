//! # Sistema de Tasks Assíncronas
//!
//! ## O que é Async/Await no Kernel?
//!
//! Async/await permite concorrência cooperativa sem threads.
//! Uma **task** é uma unidade de trabalho que pode pausar (await)
//! e retomar mais tarde, sem bloquear outras tasks.
//!
//! ## Como funciona?
//!
//! ```text
//! Task = Future + ID
//!
//! Future é uma state machine:
//! ┌──────────┐      ┌──────────┐
//! │  poll()  │ ─→─ │ Pending  │ ──┐
//! └──────────┘      └──────────┘   │ (aguarda evento)
//!      │                          │
//!      v                          v
//! ┌──────────┐      ┌──────────┐
//! │ Ready(T) │ ─←─ │  wake()  │ ──┘
//! └──────────┘      └──────────┘
//! ```
//!
//! ## Estruturas
//!
//! - **TaskId**: Identificador único gerado atomicamente
//! - **Task**: Wrapper de um Future com ID e Box pinado
//!
//! ## Por que Pin?
//!
//! Futures podem ter self-references (referências a próprios campos).
//! `Pin<Box<Future>>` garante que o Future não será movido na memória,
//! invalidando essas referências.
//!
//! ## Estudo baseado em
//!
//! [Async/Await](https://os.phil-opp.com/async-await/) - Blog OS

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

/// Identificador único de uma task.
///
/// Gerado atomicamente usando `AtomicU64` para garantir unicidade
/// mesmo em contextos concorrentes (interrupções).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    /// Gera um novo TaskId único incrementando um contador global.
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Uma task assíncrona que pode ser executada pelo Executor.
///
/// Encapsula um Future em um `Pin<Box<dyn Future>>` para:
/// - Permitir futures de qualquer tipo (`dyn Future`)
/// - Alocar no heap (`Box`) já que o tamanho é desconhecido em compile-time
/// - Prevenir movimentação (`Pin`) para self-references seguras
pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Cria uma nova task a partir de um Future.
    ///
    /// O Future deve ter lifetime `'static` pois a task pode viver
    /// indefinidamente no executor.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// Faz polling do Future, avançando sua execução.
    ///
    /// Retorna `Poll::Ready(())` quando completo ou `Poll::Pending`
    /// se aguardando algum evento externo.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

