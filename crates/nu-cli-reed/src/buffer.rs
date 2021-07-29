use crate::shell::{Completer, CompletionContext, Helper};
use nu_engine::evaluation_context::EvaluationContext;
use nu_engine::script::LineResult;

fn chomp_newline(s: &str) -> &str {
    if let Some(s) = s.strip_suffix('\n') {
        s
    } else {
        s
    }
}

/// This is a replacement of nu_engine's process_script with the exact same
/// input and output parameters
pub fn process_buffer(
    script_text: &str,
    ctx: &EvaluationContext,
    _redirect_stdin: bool,
    _span_offset: usize,
    _cli_mode: bool,
) -> LineResult {
    if script_text.trim() == "" {
        LineResult::Success(script_text.to_string())
    } else {
        let line = chomp_newline(script_text);
        println!("{}", line);

        let helper = Helper::new(ctx.clone());
        let ctx = CompletionContext(ctx);
        let suggestions = helper.complete(line, 0, &ctx);
        println!("{:?}", suggestions);
        LineResult::Success(line.to_string())
    }
}