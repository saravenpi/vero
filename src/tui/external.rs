use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use crate::models::Email;

pub fn open_editor_for_draft(editor: &str, draft_path: &Path) -> Result<()> {
    run_with_terminal_suspended(|| run_editor(editor, draft_path))
}

pub fn open_email_in_viewer(viewer: &str, email: &Email) -> Result<()> {
    let temp_file = write_temp_email(email)?;
    let result = run_with_terminal_suspended(|| run_viewer(viewer, &temp_file));
    fs::remove_file(&temp_file).ok();
    result
}

fn run_with_terminal_suspended<F>(run: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    let result = run();

    let restore_result = (|| -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(())
    })();

    restore_result?;
    result
}

fn run_editor(editor: &str, draft_path: &Path) -> Result<()> {
    let (command, args) = split_command(editor)?;
    let mut final_args = args;
    final_args.push(draft_path.to_string_lossy().to_string());
    run_command(&command, &final_args)
}

fn run_viewer(viewer: &str, temp_file: &Path) -> Result<()> {
    let (command, mut args) = split_command(viewer)?;

    if command.contains("vim") || command.contains("nvim") {
        args.push("-R".to_string());
    }

    args.push(temp_file.to_string_lossy().to_string());
    run_command(&command, &args)
}

fn split_command(command_line: &str) -> Result<(String, Vec<String>)> {
    let parts = command_line.split_whitespace().collect::<Vec<_>>();
    let (command, args) = parts.split_first().context("Command is empty")?;

    Ok((
        (*command).to_string(),
        args.iter().map(|arg| (*arg).to_string()).collect(),
    ))
}

fn run_command(command: &str, args: &[String]) -> Result<()> {
    let status = Command::new(command)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute {}", command))?;

    if !status.success() {
        anyhow::bail!("{} exited with non-zero status", command);
    }

    Ok(())
}

fn write_temp_email(email: &Email) -> Result<std::path::PathBuf> {
    let temp_file =
        std::env::temp_dir().join(format!("vero_email_{}.eml", chrono::Utc::now().timestamp()));
    let content = format_email(email);

    let mut file = fs::File::create(&temp_file).context("Failed to create temporary file")?;
    file.write_all(content.as_bytes())
        .context("Failed to write email to temporary file")?;

    Ok(temp_file)
}

fn format_email(email: &Email) -> String {
    let to = email.to.as_deref().unwrap_or("Unknown");
    let cc = email.cc.as_deref().unwrap_or("");

    format!(
        "from: {}\nto: {}\n{}subject: {}\ndate: {}\n\n{}\n",
        email.from,
        to,
        if cc.is_empty() {
            String::new()
        } else {
            format!("cc: {}\n", cc)
        },
        email.subject,
        email.date,
        email.body.replace("\r\n", "\n").replace('\r', "\n")
    )
}
