mod cancelable_background;
mod worker;

pub use {cancelable_background::*, worker::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cancelable_background::run()
}
