use crate::line_editor::create_line_editor;
use nu_engine::script::{process_script, LineResult};
use nu_engine::{maybe_print_errors, run_block, script::run_script_standalone, EvaluationContext};
use reedline::{DefaultPrompt, Signal};

#[allow(unused_imports)]
use nu_data::config;
use nu_source::{Tag, Text};
use nu_stream::InputStream;
#[allow(unused_imports)]
use std::sync::atomic::Ordering;

use nu_errors::ShellError;
use nu_parser::ParserScope;
use nu_protocol::{hir::ExternalRedirection, ConfigPath, UntaggedValue, Value};

use log::trace;
use std::error::Error;
use std::iter::Iterator;
use std::path::PathBuf;

pub fn search_paths() -> Vec<std::path::PathBuf> {
    use std::env;

    let mut search_paths = Vec::new();

    // Automatically add path `nu` is in as a search path
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            search_paths.push(exe_dir.to_path_buf());
        }
    }

    if let Ok(config) = nu_data::config::config(Tag::unknown()) {
        if let Some(Value {
            value: UntaggedValue::Table(pipelines),
            ..
        }) = config.get("plugin_dirs")
        {
            for pipeline in pipelines {
                if let Ok(plugin_dir) = pipeline.as_string() {
                    search_paths.push(PathBuf::from(plugin_dir));
                }
            }
        }
    }

    search_paths
}

pub fn run_script_file(
    context: EvaluationContext,
    options: super::app::CliOptions,
) -> Result<(), ShellError> {
    if let Some(cfg) = options.config {
        load_cfg_as_global_cfg(&context, PathBuf::from(cfg));
    } else {
        load_global_cfg(&context);
    }

    let _ = register_plugins(&context);
    //    let _ = configure_ctrl_c(&context);

    let script = options
        .scripts
        .get(0)
        .ok_or_else(|| ShellError::unexpected("Nu source code not available"))?;

    run_script_standalone(script.get_code().to_string(), options.stdin, &context, true)?;

    Ok(())
}

pub fn cli(
    context: EvaluationContext,
    options: super::app::CliOptions,
) -> Result<(), Box<dyn Error>> {
    if let Some(cfg) = options.config {
        load_cfg_as_global_cfg(&context, PathBuf::from(cfg));
    } else {
        load_global_cfg(&context);
    }

    let (skip_welcome_message, _prompt) = if let Some(cfg) = &context.configs().lock().global_config
    {
        (
            cfg.var("skip_welcome_message")
                .map(|x| x.is_true())
                .unwrap_or(false),
            cfg.var("prompt"),
        )
    } else {
        (false, None)
    };

    //Check whether dir we start in contains local cfg file and if so load it.
    load_local_cfg_if_present(&context);

    // Give ourselves a scope to work in
    context.scope.enter_scope();

    let session_text = String::new();
    let mut _line_start: usize = 0;

    if !skip_welcome_message {
        println!(
            "Welcome to Nushell {} (type 'help' for more info)",
            nu_command::commands::core_version()
        );
    }

    #[cfg(windows)]
    {
        let _ = nu_ansi_term::enable_ansi_support();
    }

    loop {
        let prompt = DefaultPrompt::new(1);
        let mut line_editor = create_line_editor().unwrap();
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::CtrlD | Signal::CtrlC => {
                line_editor.print_crlf().unwrap();
                break;
            }
            Signal::Success(buffer) => {
                if buffer.trim() == "rlh" {
                    line_editor.print_history()?;
                    continue;
                }
                let line = process_script(&buffer, &context, false, 0, true);

                match line {
                    LineResult::Success(_line) => {
                        println!("");
                        maybe_print_errors(&context, Text::from(session_text.clone()));
                    }
                    LineResult::ClearHistory => {
                        println!("got ClearHistory")
                    }

                    LineResult::Error(_line, err) => {
                        context
                            .host()
                            .lock()
                            .print_err(err, &Text::from(session_text.clone()));
                        maybe_print_errors(&context, Text::from(session_text.clone()));
                    }

                    LineResult::CtrlC => {
                        println!("got a CtrlC");
                    }

                    LineResult::CtrlD => {
                        println!("got a CtrlD");
                    }

                    LineResult::Break => {
                        break;
                    }
                }
            }

            Signal::CtrlL => {
                line_editor.clear_screen().unwrap();
            }
        }
    }
    Ok(())
}

pub fn load_local_cfg_if_present(context: &EvaluationContext) {
    trace!("Loading local cfg if present");
    match config::loadable_cfg_exists_in_dir(PathBuf::from(context.shell_manager().path())) {
        Ok(Some(cfg_path)) => {
            if let Err(err) = context.load_config(&ConfigPath::Local(cfg_path)) {
                context.host().lock().print_err(err, &Text::from(""))
            }
        }
        Err(e) => {
            //Report error while checking for local cfg file
            context.host().lock().print_err(e, &Text::from(""))
        }
        Ok(None) => {
            //No local cfg file present in start dir
        }
    }
}

fn load_cfg_as_global_cfg(context: &EvaluationContext, path: PathBuf) {
    if let Err(err) = context.load_config(&ConfigPath::Global(path)) {
        context.host().lock().print_err(err, &Text::from(""));
    }
}

pub fn load_global_cfg(context: &EvaluationContext) {
    match config::default_path() {
        Ok(path) => {
            load_cfg_as_global_cfg(context, path);
        }
        Err(e) => {
            context.host().lock().print_err(e, &Text::from(""));
        }
    }
}

pub fn register_plugins(context: &EvaluationContext) -> Result<(), ShellError> {
    if let Ok(plugins) = nu_engine::plugin::build_plugin::scan(search_paths()) {
        context.add_commands(
            plugins
                .into_iter()
                .filter(|p| !context.is_command_registered(p.name()))
                .collect(),
        );
    }

    Ok(())
}

pub fn parse_and_eval(line: &str, ctx: &EvaluationContext) -> Result<String, ShellError> {
    // FIXME: do we still need this?
    let line = if let Some(s) = line.strip_suffix('\n') {
        s
    } else {
        line
    };

    // TODO ensure the command whose examples we're testing is actually in the pipeline
    ctx.scope.enter_scope();
    let (classified_block, err) = nu_parser::parse(line, 0, &ctx.scope);
    if let Some(err) = err {
        ctx.scope.exit_scope();
        return Err(err.into());
    }

    let input_stream = InputStream::empty();

    let result = run_block(
        &classified_block,
        ctx,
        input_stream,
        ExternalRedirection::Stdout,
    );
    ctx.scope.exit_scope();

    result?.collect_string(Tag::unknown()).map(|x| x.item)
}

#[allow(dead_code)]
fn current_branch() -> String {
    #[cfg(feature = "shadow-rs")]
    {
        Some(shadow_rs::branch())
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .map(|x| format!("({})", x))
            .unwrap_or_default()
    }
    #[cfg(not(feature = "shadow-rs"))]
    {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use nu_engine::EvaluationContext;

    #[quickcheck]
    fn quickcheck_parse(data: String) -> bool {
        let (tokens, err) = nu_parser::lex(&data, 0);
        let (lite_block, err2) = nu_parser::parse_block(tokens);
        if err.is_none() && err2.is_none() {
            let context = EvaluationContext::basic();
            let _ = nu_parser::classify_block(&lite_block, &context.scope);
        }
        true
    }
}
