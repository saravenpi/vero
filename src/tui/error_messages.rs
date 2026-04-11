use anyhow::Error;

use crate::config::Account;

pub(crate) fn format_cached_inbox_error(error: &Error) -> String {
    format!("Cache error. {}", sentence(clean_error_text(error)))
}

pub(crate) fn format_inbox_error(account: Option<&Account>, error: &Error) -> String {
    let chain = error_chain_text(error);
    let normalized = chain.to_ascii_lowercase();

    if let Some(account) = account {
        if is_auth_failure(&normalized) {
            return format!("IMAP auth failed for {}. Check credentials in ~/.vero.yml.", account.imap.host);
        }

        if is_imap_connectivity_failure(&normalized) {
            return format!("Can't reach {}:{}. Check IMAP settings in ~/.vero.yml.", account.imap.host, account.imap.port);
        }

        return format!("IMAP error on {}. Check settings in ~/.vero.yml.", account.imap.host);
    }

    "IMAP error. Check settings in ~/.vero.yml.".to_string()
}

pub(crate) fn format_send_error(account: Option<&Account>, error: &Error) -> String {
    let chain = error_chain_text(error);
    let normalized = chain.to_ascii_lowercase();
    let cleaned = clean_error_text(error);

    if normalized.contains("invalid from address")
        || normalized.contains("invalid to address")
        || normalized.contains("invalid cc address")
        || normalized.contains("invalid bcc address")
        || normalized.contains("failed to read attachment")
        || normalized.contains("failed to stat attachment")
        || normalized.contains("failed to build attachment content type")
        || normalized.contains("attachment path is missing a filename")
        || normalized.contains("failed to build email")
        || normalized.contains("failed to build email with attachments")
    {
        return sentence(cleaned);
    }

    if normalized.contains("failed to spawn sent email storage task") {
        return "Sent, but couldn't save a local copy.".to_string();
    }

    if let Some(account) = account {
        if is_auth_failure(&normalized) {
            return format!("SMTP auth failed for {}. Check credentials in ~/.vero.yml.", account.smtp.host);
        }

        if is_smtp_connectivity_failure(&normalized) {
            return format!("Can't reach {}:{}. Check SMTP settings in ~/.vero.yml.", account.smtp.host, account.smtp.port);
        }

        return format!("SMTP error on {}. Check settings in ~/.vero.yml.", account.smtp.host);
    }

    "SMTP error. Check settings in ~/.vero.yml.".to_string()
}

fn error_chain_text(error: &Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}

fn clean_error_text(error: &Error) -> String {
    let mut text = error.to_string();

    loop {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("Failed to fetch emails: ") {
            text = rest.to_string();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Failed to send email: ") {
            text = rest.to_string();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Failed to login: ") {
            text = rest.to_string();
            continue;
        }
        break;
    }

    text.trim().to_string()
}

fn sentence(text: String) -> String {
    let trimmed = text.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        return String::new();
    }

    format!("{trimmed}.")
}

fn is_auth_failure(text: &str) -> bool {
    text.contains("authenticationfailed")
        || text.contains("authentication failed")
        || text.contains("invalid credentials")
        || text.contains("credentials invalid")
        || text.contains("username and password not accepted")
        || text.contains("535 5.7.8")
}

fn is_imap_connectivity_failure(text: &str) -> bool {
    text.contains("connection timeout")
        || text.contains("failed to connect to imap server")
        || text.contains("tls handshake timeout")
        || text.contains("tls connection failed")
        || text.contains("dns error")
        || text.contains("failed to lookup address information")
        || text.contains("name or service not known")
        || text.contains("connection refused")
        || text.contains("timed out")
}

fn is_smtp_connectivity_failure(text: &str) -> bool {
    text.contains("failed to create smtp transport")
        || text.contains("connection error")
        || text.contains("connection refused")
        || text.contains("timed out")
        || text.contains("tls error")
        || text.contains("dns error")
        || text.contains("failed to lookup address information")
        || text.contains("name or service not known")
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use crate::tui::test_support::test_app;

    use super::{format_inbox_error, format_send_error};

    #[test]
    fn inbox_auth_errors_get_human_message() {
        let app = test_app();
        let error =
            anyhow!("Failed to login: no response: code: None, info: Some(\"[AUTHENTICATIONFAILED] Authentication failed.\")");

        let msg = format_inbox_error(app.current_account.as_ref(), &error);

        assert_eq!(
            msg,
            "IMAP auth failed for imap.example.com. Check credentials in ~/.vero.yml."
        );
    }

    #[test]
    fn smtp_auth_errors_get_human_message() {
        let app = test_app();
        let error = anyhow!("Failed to send email: 535 5.7.8 Authentication failed");

        let msg = format_send_error(app.current_account.as_ref(), &error);

        assert_eq!(
            msg,
            "SMTP auth failed for smtp.example.com. Check credentials in ~/.vero.yml."
        );
    }

    #[test]
    fn send_validation_errors_stay_specific() {
        let error = anyhow!("Invalid to address");

        let msg = format_send_error(None, &error);

        assert_eq!(msg, "Invalid to address.");
    }
}
