use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct MessageFields {
    pub(super) headers: HashMap<String, String>,
    pub(super) body: String,
}

pub(super) fn parse_strict_fields(content: &str) -> Result<MessageFields> {
    parse_fields(content, true)
}

pub(super) fn parse_lenient_fields(content: &str) -> MessageFields {
    parse_fields(content, false).unwrap_or_default()
}

fn parse_fields(content: &str, reject_invalid_header_lines: bool) -> Result<MessageFields> {
    let mut fields = MessageFields::default();
    let mut in_body = false;
    let mut body_started = false;

    for (line_num, line) in content.lines().enumerate() {
        if in_body {
            if body_started {
                fields.body.push('\n');
            } else {
                body_started = true;
            }
            fields.body.push_str(line);
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let Some(colon_pos) = line.find(':') else {
            if reject_invalid_header_lines {
                return Err(anyhow!(
                    "Line {}: Missing ':' in header line: {}",
                    line_num + 1,
                    line
                ));
            }
            continue;
        };

        let field = line[..colon_pos].trim().to_lowercase();
        let value = line[colon_pos + 1..].trim();

        if field == "body" {
            in_body = true;
            if !value.is_empty() {
                fields.body.push_str(value);
                body_started = true;
            }
        } else {
            fields.headers.insert(field, value.to_string());
        }
    }

    Ok(fields)
}
