use clipboard::{ClipboardContext, ClipboardProvider};
use crate::errors::ClipboardError;

pub fn copy_to_clipboard(content: &str) -> Result<(), ClipboardError> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    ctx.set_contents(content.to_owned())?;
    Ok(())
}
