use crate::shell::{Completer, CompletionContext, Helper};
// use crate::shell:CompletionSuggestion;

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

        let helper = Helper::new(EvaluationContext::basic());
        let ctx = CompletionContext(ctx);
        let _suggestions = helper.complete(line, 0, &ctx);
        // let answer = suggestions.into_iter().map(CompletionSuggestion).collect();
        // println!("{:?}",suggestions);

        /*
                let ctx = CompletionContext(&self.context);
                let suggestions = self.completer.complete(line, pos, &ctx);
                let answer = suggestions.into_iter().map(CompletionSuggestion).collect();
        */

        LineResult::Success(line.to_string())
    }
}
