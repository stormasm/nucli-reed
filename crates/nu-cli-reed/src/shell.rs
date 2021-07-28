use nu_ansi_term::Color;
use nu_completion::NuCompleter;

use nu_completion::Suggestion;

use nu_engine::{DefaultPalette, EvaluationContext, Painter};
use nu_errors::ShellError;
use nu_source::{Tag, Tagged};
use std::borrow::Cow::{self, Owned};

pub struct Helper {
    completer: NuCompleter,
    context: EvaluationContext,
    pub colored_prompt: String,
}

impl Helper {
    pub(crate) fn new(context: EvaluationContext) -> Helper {
        Helper {
            completer: NuCompleter {},
            context,
            colored_prompt: String::new(),
        }
    }
}

use nu_protocol::{SignatureRegistry, VariableRegistry};
pub struct CompletionContext<'a>(pub &'a EvaluationContext);

impl<'a> nu_completion::CompletionContext for CompletionContext<'a> {
    fn signature_registry(&self) -> &dyn SignatureRegistry {
        &self.0.scope
    }

    fn scope(&self) -> &dyn nu_parser::ParserScope {
        &self.0.scope
    }

    fn source(&self) -> &EvaluationContext {
        self.as_ref()
    }

    fn variable_registry(&self) -> &dyn VariableRegistry {
        self.0
    }
}

impl<'a> AsRef<EvaluationContext> for CompletionContext<'a> {
    fn as_ref(&self) -> &EvaluationContext {
        self.0
    }
}

/// A completion candidate from rustyline this should be removed...
/// Or integrated into the code base
pub trait Candidate {
    /// Text to display when listing alternatives.
    fn display(&self) -> &str;
    /// Text to insert in line.
    fn replacement(&self) -> &str;
}

impl Candidate for String {
    fn display(&self) -> &str {
        self.as_str()
    }

    fn replacement(&self) -> &str {
        self.as_str()
    }
}

/// #[deprecated = "Unusable"]
impl Candidate for str {
    fn display(&self) -> &str {
        self
    }

    fn replacement(&self) -> &str {
        self
    }
}

impl Candidate for &'_ str {
    fn display(&self) -> &str {
        self
    }

    fn replacement(&self) -> &str {
        self
    }
}

pub struct CompletionSuggestion(Suggestion);

impl Candidate for CompletionSuggestion {
    fn display(&self) -> &str {
        &self.0.display
    }

    fn replacement(&self) -> &str {
        &self.0.replacement
    }
}

/// To be called for tab-completion.
pub trait Completer {
    /// Specific completion candidate.
    type Candidate: Candidate;

    /// Takes the currently edited `line` with the cursor `pos`ition and
    /// returns the start position and the completion candidates for the
    /// partial word to be completed.
    ///
    /// ("ls /usr/loc", 11) => Ok((3, vec!["/usr/local/"]))
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &CompletionContext<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ShellError> {
        let _ = (line, pos, ctx);
        Ok((0, Vec::with_capacity(0)))
    }
}

impl Completer for Helper {
    type Candidate = CompletionSuggestion;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &CompletionContext<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ShellError> {
        let ctx = CompletionContext(&self.context);
        let (position, suggestions) = self.completer.complete(line, pos, &ctx);
        let suggestions = suggestions.into_iter().map(CompletionSuggestion).collect();
        Ok((position, suggestions))
    }
}

impl rustyline::highlight::Highlighter for Helper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        use std::borrow::Cow::Borrowed;

        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(Color::DarkGray.prefix().to_string() + hint + nu_ansi_term::ansi::RESET)
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let cfg = &self.context.configs().lock();
        if let Some(palette) = &cfg.syntax_config {
            Painter::paint_string(line, &self.context.scope, palette)
        } else {
            Painter::paint_string(line, &self.context.scope, &DefaultPalette {})
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

struct NuValidator {}

impl rustyline::validate::Validator for NuValidator {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let src = ctx.input();

        let (tokens, err) = nu_parser::lex(src, 0, nu_parser::NewlineMode::Normal);
        if let Some(err) = err {
            if let nu_errors::ParseErrorReason::Eof { .. } = err.reason() {
                return Ok(rustyline::validate::ValidationResult::Incomplete);
            }
        }

        let (_, err) = nu_parser::parse_block(tokens);

        if let Some(err) = err {
            if let nu_errors::ParseErrorReason::Eof { .. } = err.reason() {
                return Ok(rustyline::validate::ValidationResult::Incomplete);
            }
        }

        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

#[allow(unused)]
fn vec_tag<T>(input: Vec<Tagged<T>>) -> Option<Tag> {
    let mut iter = input.iter();
    let first = iter.next()?.tag.clone();
    let last = iter.last();

    Some(match last {
        None => first,
        Some(last) => first.until(&last.tag),
    })
}

// impl rustyline::Helper for Helper {}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_engine::EvaluationContext;
    use rustyline::completion::Completer;
    use rustyline::line_buffer::LineBuffer;

    #[ignore]
    #[test]
    fn closing_quote_should_replaced() {
        let text = "cd \"folder with spaces\\subdirectory\\\"";
        let replacement = "\"folder with spaces\\subdirectory\\subsubdirectory\\\"";

        let mut buffer = LineBuffer::with_capacity(256);
        buffer.insert_str(0, text);
        buffer.set_pos(text.len() - 1);

        let helper = Helper::new(EvaluationContext::basic(), None);

        helper.update(&mut buffer, "cd ".len(), replacement);

        assert_eq!(
            buffer.as_str(),
            "cd \"folder with spaces\\subdirectory\\subsubdirectory\\\""
        );
    }

    #[ignore]
    #[test]
    fn replacement_with_cursor_in_text() {
        let text = "cd \"folder with spaces\\subdirectory\\\"";
        let replacement = "\"folder with spaces\\subdirectory\\subsubdirectory\\\"";

        let mut buffer = LineBuffer::with_capacity(256);
        buffer.insert_str(0, text);
        buffer.set_pos(text.len() - 30);

        let helper = Helper::new(EvaluationContext::basic(), None);

        helper.update(&mut buffer, "cd ".len(), replacement);

        assert_eq!(
            buffer.as_str(),
            "cd \"folder with spaces\\subdirectory\\subsubdirectory\\\""
        );
    }
}
