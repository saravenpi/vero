# Vero Email Client Upgrade Plan

## Overview

This document outlines a major architectural upgrade to Vero that transforms it into a file-based, editor-first email client. The goal is to make email composition more powerful and flexible by leveraging external editors while maintaining persistent draft storage.

## Philosophy

**Everything is a file.** This upgrade embraces the Unix philosophy by making emails, drafts, and composition workflows file-based. Users can leverage their favorite editors (vim, neovim, emacs, nano, etc.) for the full email composition experience, not just the body.

## Current Limitations

- Editor integration only works for email body (Ctrl+E during body composition)
- Email composition uses multi-step TUI forms (To → CC → Subject → Body → Preview)
- No draft persistence (abandoned compositions are lost)
- Storage is account-agnostic (flat structure in `~/.vero/seen/` and `~/.vero/sent/`)
- Cannot edit full email metadata (to, from, cc, subject) in external editor

## Proposed Changes

### 1. File-Based Email Architecture

**New Directory Structure:**
```
~/.vero/
├── <account1@example.com>/
│   ├── drafts/
│   │   └── 2025-01-15-143022.eml
│   ├── sent/
│   │   └── 2025-01-15-120000-recipient_example_com.eml
│   └── seen/
│       └── 2025-01-15-100000-sender_example_com.eml
└── <account2@example.com>/
    ├── drafts/
    ├── sent/
    └── seen/
```

**Key Points:**
- Each account gets its own directory
- Drafts are stored as they're being composed
- Sent and seen emails use the same file format
- Filenames include timestamps for sorting and uniqueness

### 2. Email File Format

All emails (drafts, sent, seen) use a simple header-body format:

```
to: recipient@example.com
cc: another@example.com, third@example.com
bcc: secret@example.com
subject: Email subject goes here
attachments: ~/Documents/report.pdf, /tmp/screenshot.png
body: First line of body can start here
Or it can start on the next line
And continue for multiple lines
Everything after "body:" is part of the email body
```

**Format Rules:**

1. **Headers must come before body**
   - Order of headers doesn't matter (except `body:` must be last)
   - Each header format: `field: value`
   - Empty lines between headers are ignored

2. **Required fields:**
   - `to:` - Must not be empty when sending
   - `subject:` - Must not be empty when sending
   - `body:` - Must be last field

3. **Optional fields:**
   - `cc:` - Comma-separated email addresses (can be empty)
   - `bcc:` - Comma-separated email addresses (can be empty)
   - `attachments:` - Comma-separated file paths (can be empty)

4. **System fields (not in draft file):**
   - `from:` - Automatically set from current account config when sending
   - The user never edits this field; it's determined by which account is active

5. **Body handling:**
   - Everything after `body:` until EOF is the email body
   - Leading and trailing whitespace is trimmed
   - Body can start on same line as `body:` or next line
   - Preserves internal formatting, line breaks, etc.

**Parsing Logic:**
- Read line by line until `body:` is encountered
- For header lines: split on first `:` to get field name and value
- Trim whitespace from field names and values
- Skip empty lines between headers
- Error on malformed header lines (no `:` character)
- Error on empty required fields (to, subject) when sending
- Ignore unknown header fields (for forward compatibility)
- Everything after `body:` line becomes the body content

**Attachment Handling:**
- Parse `attachments:` field by splitting on commas
- Trim whitespace from each path
- Expand `~` to home directory
- Validate that files exist before sending
- Error if attachment file not found or not readable
- Support both absolute paths and relative paths (relative to home or current dir)
- Empty attachment field is valid (no attachments)

### 3. Editor-First Composition

**Current Flow (to be removed):**
```
Compose Screen → To input → CC input → Subject input → Body textarea → Preview → Send
```

**New Flow:**
```
Compose Screen → Open editor immediately → Parse file → Preview → Send
```

**Implementation:**
1. When user selects "Compose" from menu:
   - Check if `config.editor` is set; if not, show error and return to menu
   - Create new draft file at `~/.vero/<account>/drafts/<timestamp>.eml`
   - Pre-populate with template (all fields empty except helpful comments)
   - Template format:
     ```
     to:
     cc:
     bcc:
     subject:
     attachments:
     body:
     ```
   - Open editor with the draft file
   - Wait for editor to close
   - Parse the draft file
   - Validate required fields and attachments
   - Show preview screen with parsed email
   - Allow send/edit/cancel

2. Editor command from config:
   - Use `config.editor` field from `~/.vero.yml`
   - Support arguments: `editor: "nvim -c 'startinsert'"`
   - Editor is REQUIRED - no fallback (show helpful error if not configured)

### 4. Draft Management

**Auto-saving (Optional Enhancement):**
- Initially, drafts are created only when entering compose screen
- Future enhancement: Keep draft file open and monitor for changes
- For now: Simple flow of create → edit → parse → send/discard

**Draft Operations:**
- Save draft: File is already saved by editor
- Resume draft: Load existing draft from drafts directory (future feature)
- Delete draft: Remove file after sending or canceling
- List drafts: Future feature to show all drafts in a "Drafts" menu item

### 5. Sent & Seen Email Storage

**Update sent email storage:**
- Use new file format instead of YAML
- Store in `~/.vero/<account>/sent/`
- Filename: `YYYY-MM-DD-HHMMSS-sanitized_to.eml`

**Update seen email storage:**
- Use new file format instead of YAML
- Store in `~/.vero/<account>/seen/`
- Filename: `YYYY-MM-DD-HHMMSS-sanitized_from.eml`

**Note:** This replaces the current YAML serialization with the simpler email format.

## Implementation Details

### Editor Requirement

**Editor is REQUIRED.** Vero is a power-user tool designed for people who live in the terminal. There is no TUI fallback for email composition.

**Behavior when editor is not configured:**
- When user selects "Compose" and `config.editor` is not set, show helpful error:
  ```
  ┌─────────────────────────────────────────┐
  │         No editor configured!           │
  │                                         │
  │  To compose emails, add an editor to    │
  │  ~/.vero.yml:                           │
  │                                         │
  │    editor: vim                          │
  │    editor: nano                         │
  │    editor: "nvim -c 'startinsert'"      │
  │    editor: emacs                        │
  │                                         │
  │  Press any key to return to menu.       │
  └─────────────────────────────────────────┘
  ```
- Return to menu immediately
- Do not create draft file
- Clear and simple: configure editor to use Vero

### Migration Strategy

**Clean break approach.** This is a major version upgrade (v2.0.0+), breaking changes are acceptable.

**Implementation:**
- Immediately use new directory structure: `~/.vero/<account>/drafts/`, `~/.vero/<account>/sent/`, `~/.vero/<account>/seen/`
- Old emails in `~/.vero/seen/` and `~/.vero/sent/` are not migrated automatically
- Users can manually move files if desired
- No automatic conversion from YAML to new format
- Clean slate, simpler implementation, no migration code to maintain

### Enhanced Draft Features (Future)

**Current scope:** Basic draft creation and editing

**Future enhancements:**
- List all drafts in a "Drafts" menu section
- Auto-save drafts when switching screens
- Resume unfinished drafts
- Draft metadata (last edited, recipient preview)

### Error Handling

**Critical errors to handle gracefully:**

1. **Editor fails to launch:** Show error, offer retry or cancel
2. **Malformed email file:** Show specific parsing errors with line numbers
3. **Missing required fields:** Highlight which fields are empty, offer re-edit
4. **Email address validation:** Warn about invalid email formats (but don't block)
5. **Concurrent file access:** Detect if draft was modified externally

**Example error message:**
```
Failed to parse email draft:

Line 3: Missing ':' in header line
Line 8: 'to' field is required but empty
Line 12: 'body' field must be the last field

Press 'e' to edit again, ESC to cancel
```

## Implementation Checklist

### Phase 1: File Format & Storage
- [ ] Create email file parser (header-body format)
- [ ] Create email file writer (convert EmailDraft to file format)
- [ ] Add per-account directory structure functions
- [ ] Update `save_sent_email()` to use new format and location
- [ ] Update `save_seen_email()` to use new format and location
- [ ] Update email loading functions to parse new format

### Phase 2: Editor Integration
- [ ] Remove old TUI compose form code (To/CC/Subject/Body steps)
- [ ] Create draft file on compose screen entry
- [ ] Implement editor launcher (reuse existing `open_editor_for_compose`)
- [ ] Add draft file parsing after editor closes
- [ ] Implement preview screen for parsed draft
- [ ] Add validation for required fields
- [ ] Add error handling and re-edit option

### Phase 3: Draft Management
- [ ] Create drafts directory on first use
- [ ] Save draft files with timestamp-based names
- [ ] Delete draft after successful send
- [ ] Keep draft on cancel (for future resume feature)

### Phase 4: Polish & Documentation
- [ ] Add editor config validation on startup
- [ ] Add helpful error screen for missing editor config
- [ ] Update help text and keybindings
- [ ] Update CLAUDE.md documentation
- [ ] Add attachment file validation
- [ ] Add helpful comments to draft template

## Testing Scenarios

- [ ] Compose email with all fields filled (to, cc, bcc, subject, attachments, body)
- [ ] Compose email with minimal fields (only to, subject, body)
- [ ] Compose email with BCC only (no CC)
- [ ] Compose email with multiple attachments
- [ ] Compose email with tilde path in attachments (`~/file.pdf`)
- [ ] Cancel composition (draft should remain)
- [ ] Send email (draft should be deleted)
- [ ] Malformed email file (missing colons, wrong order, etc.)
- [ ] Empty required fields (to, subject)
- [ ] Multi-line body content
- [ ] Special characters in email addresses and subject
- [ ] Editor with arguments in config
- [ ] No editor configured (should show helpful error)
- [ ] Multiple accounts (drafts go to correct account directory)
- [ ] Attachment file not found (should show error)
- [ ] Attachment file not readable (should show error)
- [ ] Empty attachment field (should work fine)

## Design Decisions

1. **`from` field is NOT editable**
   - Automatically set from the current account configuration
   - Prevents confusion and SMTP authentication issues
   - User never sees or edits this field in the draft

2. **BCC field is supported**
   - Optional field, comma-separated like CC
   - Empty by default

3. **Attachments are supported**
   - Optional `attachments:` header field
   - Comma-separated file paths
   - Support for `~` expansion and relative paths
   - Validation before sending

4. **Draft expiration**
   - Drafts are kept indefinitely
   - User can manually delete old drafts
   - Future feature: "Drafts" menu section to manage them

## Example: Complete Email File

**Draft file that user edits (`~/.vero/user@example.com/drafts/2025-01-15-143022.eml`):**
```
to: recipient@example.com
cc: colleague@example.com, boss@example.com
bcc: private@example.com
subject: Q1 Report and Budget Review
attachments: ~/Documents/Q1_Report.pdf, ~/Documents/Budget.xlsx
body: Hi team,

Please find attached the Q1 report and updated budget spreadsheet.

Key highlights:
- Revenue up 15% compared to Q4
- Operating costs reduced by 8%
- New client acquisitions exceeded target

Let me know if you have any questions.

Best regards,
User
```

**Note:** The `from:` field is NOT in the draft file. It's automatically added by the system when sending, based on the current account (e.g., `user@example.com`).

## Benefits of This Approach

1. **Power user friendly:** Full editor capabilities for email composition
2. **Persistent drafts:** Never lose work in progress
3. **Portable format:** Plain text files, easy to backup/sync/version control
4. **Scriptable:** Users could generate emails programmatically
5. **Transparent:** Users can see exactly what they're sending
6. **Unix philosophy:** Simple, composable, file-based tools
7. **Future-proof:** Easy to add features (templates, signatures, etc.)

