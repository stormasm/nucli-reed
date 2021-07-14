
### nu-cli-reed

I have developed a nushell cli that depends on
[reedline](https://github.com/jonathandturner/reedline) instead of [rustyline](https://github.com/kkawakam/rustyline) and developed
a nu crate called nu-cli-reed.

Every time a new release of nushell comes out I update the cli
code with the latest nu-cli crate and then remove all of the
dependencies on rustyline and add back in the reedline code.

This code is packaged as a nu crate called **nu-cli-reed**.

The Cargo.toml file keys off the new released nushell crates
along with a code snapshot of an internal reedline crate.

Because a reedline crate is not yet published I reference my own
internal reedline crate.

### History command called rlh

The main features that works is history...  There are obviously no completions as in nushell that currently depends on rustyline.

There are two histories ---- the nushell history and the reedline history [the reedline history uses a command rlh] which is a hack as I didn't want to have to modify any other crates in nushell.

### How to run the code

```rust
cargo run
```

### How to run the example code

```rust
cargo run --example reed
cargo run --example repl
```

### Integrate reedline into nushell

For the initial code we use this commit point:

### Working commit points

##### July 13, 2021, version 0.34.0 [e95d8465](https://github.com/jntrnr/reedline/commit/e95d8465)

##### June 13, 2021, version 0.33.0 [7257797](https://github.com/jonathandturner/reedline/commit/725779728c078fa62ee7b16a6589ae4cc03ee44a)
