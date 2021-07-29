use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, DefaultCompleter, DefaultCompletionActionHandler,
    DefaultHighlighter, DefaultHinter, EditCommand, FileBackedHistory, Reedline,
};
use std::error::Error;

pub fn create_line_editor() -> Result<Reedline, Box<dyn Error>> {
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Char('m'),
        vec![EditCommand::BackspaceWord],
    );

    let history = Box::new(FileBackedHistory::with_file(50, "history.txt".into())?);
    let commands = vec![
        "test".into(),
        "clear".into(),
        "exit".into(),
        "history".into(),
        "logout".into(),
        "hello world".into(),
        "hello world reedline".into(),
        "this is the reedline crate".into(),
    ];

    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 2));

    let line_editor = Reedline::new()
        .with_history(history)?
        .with_edit_mode(reedline::EditMode::Emacs)
        .with_keybindings(keybindings)
        .with_highlighter(Box::new(DefaultHighlighter::new(commands)))
        .with_completion_action_handler(Box::new(
            DefaultCompletionActionHandler::default().with_completer(completer.clone()),
        ))
        .with_hinter(Box::new(
            DefaultHinter::default()
                .with_completer(completer) // or .with_history()
                // .with_inside_line()
                .with_style(Style::new().italic().fg(Color::LightGray)),
        ));

    Ok(line_editor)
}
