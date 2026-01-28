//! # Executor Otimizado para Tasks Assíncronas
//!
//! ## O que é um Executor?
//!
//! O executor é o runtime que gerencia e executa tasks assíncronas.
//! Ele faz polling das tasks quando acordadas e as remove quando completas.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌───────────────────────────────────────┐
//! │              Executor                 │
//! ├─────────────┬────────────┬────────────┤
//! │   tasks     │ task_queue │waker_cache│
//! │ BTreeMap    │ ArrayQueue │ BTreeMap  │
//! │ ID→Task     │ [TaskId]   │ ID→Waker  │
//! └─────────────┴────────────┴────────────┘
//! ```
//!
//! ## Fluxo de Execução
//!
//! 1. `spawn()`: Adiciona task ao mapa e ID à fila
//! 2. `run()`: Loop infinito que processa tasks e dorme quando ocioso
//! 3. `run_ready_tasks()`: Faz poll de cada task na fila
//! 4. `sleep_if_idle()`: Usa HLT para economizar CPU quando não há trabalho
//!
//! ## Sistema de Wakers
//!
//! Quando uma task retorna `Pending`, ela guarda o Waker.
//! Quando um evento ocorre (ex: tecla pressionada), o handler
//! chama `wake()`, que adiciona o TaskId de volta à fila.
//!
//! ```text
//! Handler de IRQ → wake() → task_queue.push(id) → Executor processa
//! ```
//!
//! ## Por que ArrayQueue?
//!
//! `ArrayQueue` do crossbeam é lock-free e pode ser usado em handlers
//! de interrupção sem causar deadlocks.
//!
//! ## Estudo baseado em
//!
//! [Async/Await](https://os.phil-opp.com/async-await/) - Blog OS

use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts::{self, enable_and_hlt};

/// Waker customizado que recoloca a task na fila quando acordada.
///
/// Cada task tem seu próprio TaskWaker que conhece o TaskId
/// e tem uma referência à fila compartilhada.
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    /// Adiciona o TaskId de volta à fila para ser processado.
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }

    /// Cria um novo Waker para a task especificada.
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

/// Implementação da trait Wake para integrar com o sistema de futures.
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

/// Executor otimizado que só processa tasks prontas.
///
/// Mantém um cache de Wakers para evitar recriação a cada poll.
pub struct Executor {
    /// Mapa de todas as tasks registradas (ID -> Task)
    tasks: BTreeMap<TaskId, Task>,
    /// Fila de IDs de tasks prontas para executar (lock-free)
    task_queue: Arc<ArrayQueue<TaskId>>,
    /// Cache de Wakers para reutilização
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor { 
            tasks: BTreeMap::new(), 
            task_queue: Arc::new(ArrayQueue::new(100)), 
            waker_cache: BTreeMap::new() 
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks")
        }
        self.task_queue.push(task_id).expect("queue full")
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
        
    }


    fn sleep_if_idle(&self) {
        if self.task_queue.is_empty() {
            interrupts::disable();

            if self.task_queue.is_empty() {
                enable_and_hlt();
            } else {
                interrupts::enable();
            }
        }
    }
}

