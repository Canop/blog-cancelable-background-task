use {
    crate::worker::*,
    crossbeam::channel::Receiver,
    crossterm::{
        event::{read, Event, KeyCode, KeyEvent},
        terminal,
    },
    std::fmt,
};

/// The task to execute
pub struct Task {
    id: usize,
    // probably more fields
}

#[derive(Debug)]
pub enum TaskError {
    Interrupted,
    Other, // probably more variants
}
impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Interrupted => write!(f, "task interrupted"),
            Self::Other => write!(f, "something went wrong"),
        }
    }
}
impl std::error::Error for TaskError {}

impl Task {
    pub fn execute(self, interrupt: Receiver<()>) -> Result<(), TaskError> {
        for i in 1..=4 {
            log(format!("Task {}: {}/4", self.id, i));
            if interrupt
                .recv_timeout(std::time::Duration::from_secs(1))
                .is_ok()
            {
                return Err(TaskError::Interrupted);
            }
        }
        Ok(())
    }
}

/// Loop, executing tasks while accepting user input between them
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut next_task_id = 1;
    let worker = Worker::new();
    log("Type 't' to queue a task, 'i' to interrupt the current one, anything else to exit");
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = read()
        {
            match c {
                't' => {
                    let task = Task { id: next_task_id };
                    next_task_id += 1;
                    worker.execute(task);
                }
                'i' => {
                    worker.cancel_current_task();
                }
                _ => {
                    log("Exiting");
                    break;
                }
            }
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}

/// Log a message to the terminal
///
/// This function is necessary because `println!` in a raw terminal would not
/// cleanly write to right location.
pub fn log<T: fmt::Display>(t: T) {
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::MoveToColumn(0),
        crossterm::style::Print(t),
        crossterm::style::Print('\n'),
        crossterm::cursor::MoveToColumn(0),
    );
}
