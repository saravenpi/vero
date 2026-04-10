use async_native_tls::TlsConnector;
use async_std::net::TcpStream;
use futures::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("VERO_IMAP_DEBUG_HOST").unwrap_or_else(|_| "mail.infomaniak.com".into());
    let port = env::var("VERO_IMAP_DEBUG_PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(993);
    let user = env::var("VERO_IMAP_DEBUG_USER")?;
    let password = env::var("VERO_IMAP_DEBUG_PASSWORD")?;

    println!("Connecting to {}:{}...", host, port);
    let tcp_stream = TcpStream::connect((host.as_str(), port)).await?;
    println!("TCP connected");

    let tls = TlsConnector::new();
    let tls_stream = tls.connect(&host, tcp_stream).await?;
    println!("TLS connected");

    let client = async_imap::Client::new(tls_stream);
    println!("Logging in as {}...", user);

    let mut session = client
        .login(&user, &password)
        .await
        .map_err(|e| format!("Login failed: {}", e.0))?;
    println!("Login successful");

    println!("\nSelecting INBOX...");
    let mailbox = session.select("INBOX").await?;
    println!("INBOX info:");
    println!("  - exists: {}", mailbox.exists);
    println!("  - recent: {}", mailbox.recent);
    println!("  - unseen: {:?}", mailbox.unseen);
    println!("  - uid_validity: {:?}", mailbox.uid_validity);
    println!("  - uid_next: {:?}", mailbox.uid_next);
    println!("  - flags: {:?}", mailbox.flags);

    println!("\n--- Testing SEARCH ALL ---");
    let all_uids = session.uid_search("ALL").await?;
    println!(
        "SEARCH ALL returned {} UIDs: {:?}",
        all_uids.len(),
        all_uids
    );

    println!("\n--- Testing SEARCH UNSEEN ---");
    let unseen_uids = session.uid_search("UNSEEN").await?;
    println!(
        "SEARCH UNSEEN returned {} UIDs: {:?}",
        unseen_uids.len(),
        unseen_uids
    );

    println!("\n--- Testing SEARCH SEEN ---");
    let seen_uids = session.uid_search("SEEN").await?;
    println!(
        "SEARCH SEEN returned {} UIDs: {:?}",
        seen_uids.len(),
        seen_uids
    );

    if let Some(first_uid) = all_uids.iter().next().copied() {
        println!("\n--- Fetching first email details ---");
        let mut messages = session
            .uid_fetch(first_uid.to_string(), "ENVELOPE FLAGS")
            .await?;

        if let Some(Ok(fetch)) = messages.next().await {
            if let Some(envelope) = fetch.envelope() {
                let subject = envelope
                    .subject
                    .as_ref()
                    .and_then(|s| String::from_utf8(s.to_vec()).ok())
                    .unwrap_or_default();
                println!("First email subject: {}", subject);
                let flags: Vec<_> = fetch.flags().collect();
                println!("Flags: {:?}", flags);
            }
        }
    }

    println!("\n--- Listing available folders ---");
    let folders = session.list(None, Some("*")).await?;
    let folder_list: Vec<_> = folders.collect().await;
    println!("Found {} folders:", folder_list.len());
    for folder in folder_list.into_iter().flatten() {
        println!("  - {}", folder.name());
    }

    session.logout().await?;
    println!("\nDone!");

    Ok(())
}
