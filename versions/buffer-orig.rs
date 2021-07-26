use nu_engine::{evaluate::internal::InternalIterator, run_block, shell::CdArgs};
use nu_engine::{BufCodecReader, MaybeTextCodec, StringOrBinary};
use nu_path::canonicalize;
use nu_protocol::hir::{
    Call, ClassifiedCommand, Expression, ExternalRedirection, InternalCommand, Literal,
    NamedArguments, SpannedExpression,
};
use nu_protocol::{Primitive, UntaggedValue, Value};
use nu_stream::{InputStream, IntoInputStream};

use log::{debug, trace};
use nu_source::{AnchorLocation, Span, Tag, Tagged};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::{io::BufReader, iter::Iterator};

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
    redirect_stdin: bool,
    span_offset: usize,
    cli_mode: bool,
) -> LineResult {
    if script_text.trim() == "" {
        LineResult::Success(script_text.to_string())
    } else {
        let line = chomp_newline(script_text);

        let (block, err) = nu_parser::parse(line, span_offset, &ctx.scope);

        debug!("{:#?}", block);
        //println!("{:#?}", pipeline);

        if let Some(failure) = err {
            return LineResult::Error(line.to_string(), failure.into());
        }

        // There's a special case to check before we process the pipeline:
        // If we're giving a path by itself
        // ...and it's not a command in the path
        // ...and it doesn't have any arguments
        // ...and we're in the CLI
        // ...then change to this directory
        if cli_mode
            && block.block.len() == 1
            && block.block[0].pipelines.len() == 1
            && block.block[0].pipelines[0].list.len() == 1
        {
            if let ClassifiedCommand::Internal(InternalCommand {
                ref name,
                ref args,
                name_span,
            }) = block.block[0].pipelines[0].list[0]
            {
                let internal_name = name;
                let name = args
                    .positional
                    .as_ref()
                    .and_then(|positionals| {
                        positionals.get(0).map(|e| {
                            if let Expression::Literal(Literal::String(ref s)) = e.expr {
                                &s
                            } else {
                                ""
                            }
                        })
                    })
                    .unwrap_or("");

                ctx.sync_path_to_env();
                if internal_name == "run_external"
                    && args
                        .positional
                        .as_ref()
                        .map(|v| v.len() == 1)
                        .unwrap_or(true)
                    && args
                        .named
                        .as_ref()
                        .map(NamedArguments::is_empty)
                        .unwrap_or(true)
                    && canonicalize(ctx.shell_manager().path(), name).is_ok()
                    && Path::new(&name).is_dir()
                    && !ctx.host().lock().is_external_cmd(name)
                {
                    let tag = Tag {
                        anchor: Some(AnchorLocation::Source(line.into())),
                        span: name_span,
                    };
                    let path = {
                        // Here we work differently if we're in Windows because of the expected Windows behavior
                        #[cfg(windows)]
                        {
                            if name.ends_with(':') {
                                // This looks like a drive shortcut. We need to a) switch drives and b) go back to the previous directory we were viewing on that drive
                                // But first, we need to save where we are now
                                let current_path = ctx.shell_manager().path();

                                let split_path: Vec<_> = current_path.split(':').collect();
                                if split_path.len() > 1 {
                                    ctx.windows_drives_previous_cwd()
                                        .lock()
                                        .insert(split_path[0].to_string(), current_path);
                                }

                                let name = name.to_uppercase();
                                let new_drive: Vec<_> = name.split(':').collect();

                                if let Some(val) =
                                    ctx.windows_drives_previous_cwd().lock().get(new_drive[0])
                                {
                                    val.to_string()
                                } else {
                                    format!("{}\\", name.to_string())
                                }
                            } else {
                                name.to_string()
                            }
                        }
                        #[cfg(not(windows))]
                        {
                            name.to_string()
                        }
                    };

                    let cd_args = CdArgs {
                        path: Some(Tagged {
                            item: PathBuf::from(path),
                            tag: tag.clone(),
                        }),
                    };

                    return match ctx.shell_manager().cd(cd_args, tag) {
                        Err(e) => LineResult::Error(line.to_string(), e),
                        Ok(stream) => {
                            let iter = InternalIterator {
                                context: ctx.clone(),
                                leftovers: InputStream::empty(),
                                input: stream,
                            };
                            for _ in iter {
                                //nullopt, commands are run by iterating over iter
                            }
                            LineResult::Success(line.to_string())
                        }
                    };
                }
            }
        }

        let input_stream = if redirect_stdin {
            let file = std::io::stdin();
            let buf_reader = BufReader::new(file);
            let buf_codec = BufCodecReader::new(buf_reader, MaybeTextCodec::default());
            let stream = buf_codec.map(|line| {
                if let Ok(line) = line {
                    let primitive = match line {
                        StringOrBinary::String(s) => Primitive::String(s),
                        StringOrBinary::Binary(b) => Primitive::Binary(b.into_iter().collect()),
                    };

                    Ok(Value {
                        value: UntaggedValue::Primitive(primitive),
                        tag: Tag::unknown(),
                    })
                } else {
                    panic!("Internal error: could not read lines of text from stdin")
                }
            });
            stream.into_input_stream()
        } else {
            InputStream::empty()
        };

        trace!("{:#?}", block);

        let result = run_block(&block, ctx, input_stream, ExternalRedirection::None);

        match result {
            Ok(input) => {
                // Running a pipeline gives us back a stream that we can then
                // work through. At the top level, we just want to pull on the
                // values to compute them.

                let autoview_cmd = ctx
                    .get_command("autoview")
                    .expect("Could not find autoview command");

                if let Ok(mut output_stream) = ctx.run_command(
                    autoview_cmd,
                    Tag::unknown(),
                    Call::new(
                        Box::new(SpannedExpression::new(
                            Expression::string("autoview".to_string()),
                            Span::unknown(),
                        )),
                        Span::unknown(),
                    ),
                    input,
                ) {
                    loop {
                        match output_stream.next() {
                            Some(Value {
                                value: UntaggedValue::Error(e),
                                ..
                            }) => return LineResult::Error(line.to_string(), e),
                            Some(_item) => {
                                if ctx.ctrl_c().load(Ordering::SeqCst) {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                }

                LineResult::Success(line.to_string())
            }
            Err(err) => LineResult::Error(line.to_string(), err),
        }
    }
}
