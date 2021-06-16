use nu_engine::EvaluationContext;
use std::error::Error;

#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use nu_engine::script::LineResult;

#[cfg(feature = "rustyline-support")]
use crate::keybinding::{convert_keyevent, KeyCode};

#[cfg(feature = "rustyline-support")]
use crate::shell::Helper;

#[cfg(feature = "rustyline-support")]
use rustyline::{
    self,
    config::Configurer,
    config::{ColorMode, CompletionType, Config},
    error::ReadlineError,
    line_buffer::LineBuffer,
    At, Cmd, ConditionalEventHandler, Editor, EventHandler, Modifiers, Movement, Word,
};

#[cfg(feature = "rustyline-support")]
pub fn convert_rustyline_result_to_string(input: Result<String, ReadlineError>) -> LineResult {
    match input {
        Ok(s) if s == "history -c" || s == "history --clear" => LineResult::ClearHistory,
        Ok(s) => LineResult::Success(s),
        Err(ReadlineError::Interrupted) => LineResult::CtrlC,
        Err(ReadlineError::Eof) => LineResult::CtrlD,
        Err(err) => {
            outln!("Error: {:?}", err);
            LineResult::Break
        }
    }
}

#[derive(Clone)]
#[cfg(feature = "rustyline-support")]
struct PartialCompleteHintHandler;

#[cfg(feature = "rustyline-support")]
impl ConditionalEventHandler for PartialCompleteHintHandler {
    fn handle(
        &self,
        _evt: &rustyline::Event,
        _n: rustyline::RepeatCount,
        _positive: bool,
        ctx: &rustyline::EventContext,
    ) -> Option<Cmd> {
        Some(match ctx.hint_text() {
            Some(hint_text) if ctx.pos() == ctx.line().len() => {
                let mut line_buffer = LineBuffer::with_capacity(hint_text.len());
                line_buffer.update(hint_text, 0);
                line_buffer.move_to_next_word(At::AfterEnd, Word::Vi, 1);

                let text = hint_text[0..line_buffer.pos()].to_string();

                Cmd::Insert(1, text)
            }
            _ => Cmd::Move(Movement::ForwardWord(1, At::AfterEnd, Word::Vi)),
        })
    }
}

#[cfg(feature = "rustyline-support")]
pub fn default_rustyline_editor_configuration() -> Editor<Helper> {
    #[cfg(windows)]
    const DEFAULT_COMPLETION_MODE: CompletionType = CompletionType::Circular;
    #[cfg(not(windows))]
    const DEFAULT_COMPLETION_MODE: CompletionType = CompletionType::List;

    let config = Config::builder()
        .check_cursor_position(true)
        .color_mode(ColorMode::Forced)
        .build();
    let mut rl: Editor<_> = Editor::with_config(config);

    // add key bindings to move over a whole word with Ctrl+ArrowLeft and Ctrl+ArrowRight
    //M modifier, E KeyEvent, K KeyCode
    rl.bind_sequence(
        convert_keyevent(KeyCode::Left, Some(Modifiers::CTRL)),
        Cmd::Move(Movement::BackwardWord(1, Word::Vi)),
    );

    rl.bind_sequence(
        convert_keyevent(KeyCode::Right, Some(Modifiers::CTRL)),
        EventHandler::Conditional(Box::new(PartialCompleteHintHandler)),
    );

    // workaround for multiline-paste hang in rustyline (see https://github.com/kkawakam/rustyline/issues/202)
    rl.bind_sequence(
        convert_keyevent(KeyCode::BracketedPasteStart, None),
        rustyline::Cmd::Noop,
    );
    // Let's set the defaults up front and then override them later if the user indicates
    // defaults taken from here https://github.com/kkawakam/rustyline/blob/2fe886c9576c1ea13ca0e5808053ad491a6fe049/src/config.rs#L150-L167
    rl.set_max_history_size(100);
    rl.set_history_ignore_dups(true);
    rl.set_history_ignore_space(false);
    rl.set_completion_type(DEFAULT_COMPLETION_MODE);
    rl.set_completion_prompt_limit(100);
    rl.set_keyseq_timeout(-1);
    rl.set_edit_mode(rustyline::config::EditMode::Emacs);
    rl.set_auto_add_history(false);
    rl.set_bell_style(rustyline::config::BellStyle::default());
    rl.set_color_mode(rustyline::ColorMode::Enabled);
    rl.set_tab_stop(8);

    if let Err(e) = crate::keybinding::load_keybindings(&mut rl) {
        println!("Error loading keybindings: {:?}", e);
    }

    rl
}

pub fn configure_ctrl_c(_context: &EvaluationContext) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "ctrlc")]
    {
        let cc = _context.ctrl_c.clone();

        ctrlc::set_handler(move || {
            cc.store(true, Ordering::SeqCst);
        })?;

        if _context.ctrl_c.load(Ordering::SeqCst) {
            _context.ctrl_c.store(false, Ordering::SeqCst);
        }
    }

    Ok(())
}
