use nu_engine::EvaluationContext;
//use nu_errors::ShellError;
use std::error::Error;

#[allow(unused_imports)]
use std::sync::atomic::Ordering;

#[allow(unused_imports)]
use nu_engine::script::LineResult;

pub fn configure_ctrl_c(_context: &EvaluationContext) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "ctrlc")]
    {
        let cc = _context.ctrl_c().clone();

        ctrlc::set_handler(move || {
            cc.store(true, Ordering::SeqCst);
        })?;

        if _context.ctrl_c().load(Ordering::SeqCst) {
            _context.ctrl_c().store(false, Ordering::SeqCst);
        }
    }

    Ok(())
}
