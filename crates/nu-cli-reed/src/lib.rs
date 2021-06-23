#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod app;
mod cli;
pub use crate::cli::cli;

pub use crate::app::App;
pub use crate::cli::{parse_and_eval, register_plugins, run_script_file};

pub use nu_command::{
    commands::NuSignature as Nu, commands::Version as NuVersion, create_default_context,
};
pub use nu_data::config;
pub use nu_data::dict::TaggedListBuilder;
pub use nu_data::primitive;
pub use nu_data::value;
pub use nu_stream::{ActionStream, InputStream, InterruptibleStream};
pub use nu_value_ext::ValueExt;
pub use num_traits::cast::ToPrimitive;

// TODO: Temporary redirect
pub use nu_protocol::{did_you_mean, TaggedDictBuilder};
