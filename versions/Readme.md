
```rust
pub fn process_buffer(
    script_text: &str,
    ctx: &EvaluationContext,
    redirect_stdin: bool,
    span_offset: usize,
    cli_mode: bool,
) -> LineResult {
```

The idea is to be able to swap out the call to process_script with other methods that have the same interface...

##### buffer-orig.rs
This is the original version of porting nu-engine/script.rs over to this new file which enables testing of the completer and other things related to this trait / api.
