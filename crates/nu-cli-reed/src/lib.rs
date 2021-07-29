pub mod app;
mod cli;
pub use crate::cli::cli;
mod line_editor;

pub use crate::app::App;
pub use crate::cli::{parse_and_eval, register_plugins, run_script_file};

pub use nu_command::create_default_context;
