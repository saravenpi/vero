use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::cli::types::DraftCommand;

use super::super::ArgCursor;

pub(in crate::cli::parser) fn parse_draft(args: &mut ArgCursor) -> Result<DraftCommand> {
    match args.peek() {
        None => Ok(DraftCommand::Template { output_path: None }),
        Some("template") => {
            args.next();
            let mut output_path = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_path = Some(PathBuf::from(args.value("--output")?));
                    }
                    other => return Err(anyhow!("Unknown draft option '{}'", other)),
                }
            }

            Ok(DraftCommand::Template { output_path })
        }
        Some(other) => Err(anyhow!("Unknown draft command '{}'", other)),
    }
}
