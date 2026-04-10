use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;

use crate::email_file::{self, ParsedEmail};

pub fn parse_draft_input(path: &Path) -> Result<ParsedEmail> {
    let content = read_text_input(path)?;
    let parsed = email_file::parse_email_file(&content)?;
    email_file::validate_attachments(&parsed.attachment_paths)?;
    Ok(parsed)
}

pub fn read_text_input(path: &Path) -> Result<String> {
    if path == Path::new("-") {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        return Ok(buffer);
    }

    std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))
}

pub fn create_template(signature: Option<&str>) -> String {
    email_file::create_draft_template(signature)
}

pub fn write_template(path: &Path) -> Result<()> {
    std::fs::write(path, create_template(None))
        .with_context(|| format!("Failed to write {}", path.display()))
}
