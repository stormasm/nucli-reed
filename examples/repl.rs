use reedline::{DefaultPrompt, Reedline, Signal};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut line_editor = Reedline::new();
    let prompt = DefaultPrompt::new(1);

    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::CtrlD | Signal::CtrlC => {
                line_editor.print_crlf().unwrap();
                break;
            }
            Signal::Success(buffer) => {
                println!("We processed: {}", buffer);
            }
            Signal::CtrlL => {
                line_editor.clear_screen().unwrap();
            }
        }
    }
    Ok(())
}
