use {
    crate::*,
    crossbeam::channel::{self, select, Sender},
    std::thread,
};

/// Maximum number of tasks that can be queued. If this limit is reached, no new task will be queued.
///
/// If you don't want ro drop tasks, use `channel::unbounded` instead of `channed::bounded` when creating `s_task`
const MAX_QUEUED_TASKS: usize = 3;

/// Manage a thread to execute tasks
pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    s_die: Option<Sender<()>>,
    s_cancel: Sender<()>,
    s_task: Sender<Task>,
}
impl Worker {
    pub fn new() -> Self {
        let (s_task, r_task) = channel::bounded::<Task>(MAX_QUEUED_TASKS);
        let (s_die, r_die) = channel::bounded(1);
        let (s_cancel, r_cancel) = channel::bounded(1);
        let thread = thread::spawn(move || {
            loop {
                select! {
                    recv(r_die) -> _ => {
                        log("worker thread is stopping");
                        break;
                    }
                    recv(r_task) -> ps => {
                        match ps {
                            Ok(task) => {
                                if !r_die.is_empty() { continue; }
                                let _ = r_cancel.try_recv(); // clean any unconsumed cancel
                                match task.execute(r_cancel.clone()) {
                                    Ok(()) => {
                                        log("Task done");
                                    }
                                    Err(TaskError::Interrupted) => {
                                        log("Task interrupted");
                                    }
                                    Err(e) => {
                                        log(format!("Task error: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                log(format!("Channel error: {}", e)); // unlikely
                                break;
                            }
                        }
                    }
                }
            }
        });
        Self {
            thread: Some(thread),
            s_die: Some(s_die),
            s_task,
            s_cancel,
        }
    }
    /// Request a task execution, unless too many of them are already queued
    pub fn execute(&self, task: Task) {
        if self.s_task.try_send(task).is_ok() {
            log("Queued task");
        } else {
            log("Too many tasks in the queue, dropping one");
        }
    }
    /// Request the current task to be interrupted
    pub fn cancel_current_task(&self) {
        let _ = self.s_cancel.try_send(());
    }
    /// Make the worker stop
    /// (interrupting the current task if any)
    pub fn die(&mut self) {
        self.cancel_current_task(); // interrupt current task if any
        if let Some(sender) = self.s_die.take() {
            let _ = sender.send(());
        }
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log("child_thread.join() failed"); // should not happen
            } else {
                log("Worker gracefully stopped");
            }
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.die();
    }
}
