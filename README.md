
#### Update

So we are now successfully talking to nushell.   
We are sending nushell commands and it is responding back and writing its output to the reedline terminal.  We figured out that we need an extra println! macro in there to flush the cache for certain commands:

* version | get version
* date now

Those commands do not return a carriage return in their output whereas the other commands do...

There are four crates here:

* nu-cli the original crate out of the box
* nu-cli-rusty the bare bones slimmed down crate using [rustyline](https://github.com/kkawakam/rustyline)
* nu-cli-reed the brand new crate using [reedline](https://github.com/jonathandturner/reedline)
* reedline is here temporarily until a crate is published

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
