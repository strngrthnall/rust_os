//! # Executor Simples (Educação)
//!
//! ## O que é este módulo?
//!
//! Um executor minimalista para entender os conceitos básicos.
//! **NÃO use em produção** - use `executor.rs` que é otimizado.
//!
//! ## Problemas deste executor
//!
//! 1. **Busy-loop**: Consome 100% CPU mesmo sem trabalho
//! 2. **Dummy Waker**: Não acorda tasks (ignora wake())
//! 3. **Não escala**: Re-pollea TODAS as tasks em cada iteração
//!
//! ## Por que existe?
//!
//! Para fins didáticos! Mostra a estrutura mínima de um executor:
//!
//! ```text
//! loop {
//!     para cada task {
//!         poll(task)
//!         se Pending: recoloca na fila
//!         se Ready: remove
//!     }
//! }
//! ```
//!
//! ## O que é um Dummy Waker?
//!
//! Um Waker que não faz nada. A API de futures requer um Waker,
//! mas aqui ignoramos ele pois fazemos poll de todas as tasks
//! continuamente.
//!
//! ## Estudo baseado em
//!
//! [Async/Await](https://os.phil-opp.com/async-await/) - Blog OS

use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// Executor simples com fila FIFO de tasks.
///
/// Processa tasks em ordem, recolocando as pendentes no final da fila.
pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    /// Cria um novo executor com fila vazia.
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    /// Adiciona uma task à fila de execução.
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    /// Executa todas as tasks até completarem.
    ///
    /// **Atenção**: Este método usa busy-loop! Tasks que nunca
    /// completam farão o executor rodar para sempre.
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // Task completa, não recoloca
                Poll::Pending => self.task_queue.push_back(task), // Recoloca no final
            }
        }
    }
}

/// Cria um RawWaker que não faz nada.
///
/// RawWaker é a interface de baixo nível para Wakers.
/// Requer uma VTable com funções para clone, wake, wake_by_ref e drop.
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

/// Cria um Waker que ignora todas as operações.
///
/// # Safety
/// `Waker::from_raw` é unsafe mas nosso RawWaker é seguro
/// pois todas as funções são no-ops.
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}