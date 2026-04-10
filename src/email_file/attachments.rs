pub(super) fn parse_attachment_paths(attachments: &str) -> Vec<String> {
    if attachments.trim().is_empty() {
        return Vec::new();
    }

    attachments
        .split(',')
        .map(|path| expand_attachment_path(path.trim()))
        .collect()
}

fn expand_attachment_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}
