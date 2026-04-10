use anyhow::{anyhow, Context, Result};
use std::collections::VecDeque;

pub(in crate::cli::parser) fn parse_u32(value: &str, label: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .with_context(|| format!("Invalid {} '{}'", label, value))
}

pub(in crate::cli::parser) fn parse_usize(value: &str, label: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .with_context(|| format!("Invalid {} '{}'", label, value))
}

pub(in crate::cli::parser) fn ensure_no_args(args: &ArgCursor) -> Result<()> {
    if let Some(arg) = args.peek() {
        return Err(anyhow!("Unexpected argument '{}'", arg));
    }

    Ok(())
}

pub(in crate::cli::parser) struct ArgCursor {
    args: VecDeque<String>,
}

impl ArgCursor {
    pub(in crate::cli::parser) fn new(args: Vec<String>) -> Self {
        Self { args: args.into() }
    }

    pub(in crate::cli::parser) fn peek(&self) -> Option<&str> {
        self.args.front().map(String::as_str)
    }

    pub(in crate::cli::parser) fn next(&mut self) -> Option<String> {
        self.args.pop_front()
    }

    pub(in crate::cli::parser) fn value(&mut self, flag: &str) -> Result<String> {
        self.next()
            .ok_or_else(|| anyhow!("Missing value for {}", flag))
    }

    pub(in crate::cli::parser) fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}
