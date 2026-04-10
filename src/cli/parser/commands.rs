mod draft;
mod inbox;
mod send;
mod sent;

use anyhow::{anyhow, Result};

use super::{ensure_no_args, ArgCursor};

pub(super) use draft::parse_draft;
pub(super) use inbox::parse_inbox;
pub(super) use send::parse_send;
pub(super) use sent::parse_sent;

pub(super) fn parse_accounts(args: &mut ArgCursor) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }

    match args.next().as_deref() {
        Some("list") => ensure_no_args(args),
        Some(other) => Err(anyhow!("Unknown accounts command '{}'", other)),
        None => Ok(()),
    }
}
