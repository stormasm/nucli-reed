
There are three crates here:

* the original nu-cli crate out of the box
* the bare bones slimmed down nu-cli-rusty crate using rustyline
* the brand new nu-cli-reed crate using reedline

src/main.rs is the original out of the box main.rs that works modified
with all 3 crates simply swap out nu_cli for {nu_cli_reed, nu_cli_rusty}

Along with :

* examples/rusty.rs
* examples/reed.rs

### Integrate reedline into nushell

Use [reedline](https://github.com/jonathandturner/reedline) and replace rustyline...

### Working commit points

##### June 13, 2021

[7257797](https://github.com/jonathandturner/reedline/commit/725779728c078fa62ee7b16a6589ae4cc03ee44a)

### Build Details

So every time a new release of nushell comes out
I will branch off main and create a build.

There are 2 crates in this repo...

  * nu-cli
  * reedline

Cargo.toml keys off the release builds.
