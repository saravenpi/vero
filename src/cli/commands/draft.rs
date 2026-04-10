use anyhow::Result;

use crate::cli::output;
use crate::cli::types::{DraftCommand, OutputFormat};
use crate::services;

pub(super) fn execute(output: OutputFormat, command: DraftCommand) -> Result<()> {
    match command {
        DraftCommand::Template { output_path } => {
            if let Some(path) = output_path.as_ref() {
                services::write_template(path)?;
                output::print_template(output, "", Some(path))
            } else {
                let template = services::create_template(None);
                output::print_template(output, &template, None)
            }
        }
    }
}
