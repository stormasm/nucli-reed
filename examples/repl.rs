use crossterm::Result;
use reedline::{DefaultPrompt, Reedline, Signal, DEFAULT_PROMPT_COLOR, DEFAULT_PROMPT_INDICATOR};

fn main() -> Result<()> {
    let mut line_editor = Reedline::new();
    let prompt = DefaultPrompt::new(DEFAULT_PROMPT_COLOR, DEFAULT_PROMPT_INDICATOR, 1);

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
