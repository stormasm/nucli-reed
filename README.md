
There are three crates here:

* nu-cli the original crate out of the box
* nu-cli-rusty the bare bones slimmed down crate using [rustyline](https://github.com/kkawakam/rustyline)
* nu-cli-reed the brand new crate using [reedline](https://github.com/jonathandturner/reedline)

src/main.rs is the original out of the box main.rs that works modified
with all 3 crates simply swap out nu_cli for {nu_cli_reed, nu_cli_rusty}

Along with :

* examples/rusty.rs
* examples/reed.rs

### Build Details

So every time a new release of nushell comes out I will do a branch off main and update the code coinciding with the release.

There are 3 crates in this repo...

  * nu-cli
  * nu-cli-rusty
  * nu-cli-reed

Cargo.toml keys off the release builds using the following syntax:

```rust
nu-command = "0.32.0"
nu-data = "0.32.0"
nu-engine = "0.32.0"
```

### Integrate reedline into nushell

For the initial code we use this commit point:

### Working commit points

##### June 13, 2021

[7257797](https://github.com/jonathandturner/reedline/commit/725779728c078fa62ee7b16a6589ae4cc03ee44a)
