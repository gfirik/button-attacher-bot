//! Admin command handlers for bot statistics and user management.

use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db::Analytics;
use crate::Config;

/// Check if a user is an admin.
fn is_admin(user_id: i64, config: &Config) -> bool {
    config.admin_user_ids.contains(&user_id)
}

/// Handle the /stats command - show overall bot statistics.
pub async fn handle_stats(
    bot: Bot,
    msg: Message,
    config: Config,
    analytics: Analytics,
) -> crate::HandlerResult {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !is_admin(user_id, &config) {
        bot.send_message(msg.chat.id, "You are not authorized to use admin commands.")
            .await?;
        return Ok(());
    }

    let stats = match analytics.get_bot_stats() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to get bot stats: {}", e);
            bot.send_message(msg.chat.id, "Failed to retrieve statistics.")
                .await?;
            return Ok(());
        }
    };

    let style_str = stats
        .most_popular_style
        .as_deref()
        .unwrap_or("default");

    let top_chats: String = if stats.top_destination_chats.is_empty() {
        "None yet".to_string()
    } else {
        stats
            .top_destination_chats
            .iter()
            .take(3)
            .map(|(_id, title, count)| format!("  {} ({} posts)", title, count))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let message = format!(
        r#"<b>Bot Statistics</b>

<b>Users</b>
Total: {}
Active (24h): {}
Active (7d): {}
Active (30d): {}

<b>Publications</b>
Total: {}
Last 24h: {}
Last 7d: {}

<b>Buttons</b>
Total configured: {}
Most popular style: {}

<b>Top Destination Chats</b>
{}"#,
        stats.total_users,
        stats.active_users_24h,
        stats.active_users_7d,
        stats.active_users_30d,
        stats.total_publications,
        stats.publications_24h,
        stats.publications_7d,
        stats.total_buttons_configured,
        style_str,
        top_chats
    );

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

/// Handle the /users command - list users with pagination.
pub async fn handle_users(
    bot: Bot,
    msg: Message,
    config: Config,
    analytics: Analytics,
) -> crate::HandlerResult {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !is_admin(user_id, &config) {
        bot.send_message(msg.chat.id, "You are not authorized to use admin commands.")
            .await?;
        return Ok(());
    }

    // Parse page number from command text
    let text = msg.text().unwrap_or("");
    let page: usize = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);

    let per_page = 10;
    let offset = (page - 1) * per_page;

    let users = match analytics.get_users(per_page, offset) {
        Ok(u) => u,
        Err(e) => {
            log::error!("Failed to get users: {}", e);
            bot.send_message(msg.chat.id, "Failed to retrieve users.")
                .await?;
            return Ok(());
        }
    };

    let total_users = analytics.get_user_count().unwrap_or(0);
    let total_pages = ((total_users as usize + per_page - 1) / per_page).max(1);

    if users.is_empty() {
        bot.send_message(msg.chat.id, "No users found.")
            .await?;
        return Ok(());
    }

    let mut message = format!(
        "<b>Users (Page {}/{})</b>\n\n",
        page, total_pages
    );

    for user in &users {
        let username_str = user
            .username
            .as_ref()
            .map(|u| format!("@{}", u))
            .unwrap_or_else(|| "no username".to_string());

        let name = user
            .first_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        message.push_str(&format!(
            "<code>{}</code> {} ({})\n  Messages: {} | Publications: {}\n  Last seen: {}\n\n",
            user.user_id,
            name,
            username_str,
            user.total_messages,
            user.total_publications,
            &user.last_seen_at[..16], // Trim to date + time
        ));
    }

    if page < total_pages {
        message.push_str(&format!("\nUse <code>/users {}</code> for next page", page + 1));
    }

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

/// Handle the /broadcast command - send a message to all users.
pub async fn handle_broadcast(
    bot: Bot,
    msg: Message,
    config: Config,
    analytics: Analytics,
) -> crate::HandlerResult {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !is_admin(user_id, &config) {
        bot.send_message(msg.chat.id, "You are not authorized to use admin commands.")
            .await?;
        return Ok(());
    }

    // Get message text after /broadcast
    let text = msg.text().unwrap_or("");
    let broadcast_text = text
        .strip_prefix("/broadcast")
        .map(|s| s.trim())
        .unwrap_or("");

    if broadcast_text.is_empty() {
        bot.send_message(
            msg.chat.id,
            "Usage: /broadcast <message>\n\nThis will send the message to all users.",
        )
        .await?;
        return Ok(());
    }

    // Get all users (in batches to avoid memory issues)
    let mut total_sent = 0;
    let mut total_failed = 0;
    let mut offset = 0;
    let batch_size = 50;

    bot.send_message(msg.chat.id, "Starting broadcast...")
        .await?;

    loop {
        let users = match analytics.get_users(batch_size, offset) {
            Ok(u) => u,
            Err(e) => {
                log::error!("Failed to get users for broadcast: {}", e);
                break;
            }
        };

        if users.is_empty() {
            break;
        }

        for user in &users {
            match bot
                .send_message(ChatId(user.user_id), broadcast_text)
                .await
            {
                Ok(_) => total_sent += 1,
                Err(e) => {
                    log::warn!("Failed to send to user {}: {}", user.user_id, e);
                    total_failed += 1;
                }
            }

            // Small delay to avoid rate limits
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        offset += batch_size;
    }

    bot.send_message(
        msg.chat.id,
        format!(
            "Broadcast complete!\nSent: {}\nFailed: {}",
            total_sent, total_failed
        ),
    )
    .await?;

    Ok(())
}

/// Handle the /block command - block a user from using the bot.
pub async fn handle_block(
    bot: Bot,
    msg: Message,
    config: Config,
    analytics: Analytics,
) -> crate::HandlerResult {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !is_admin(user_id, &config) {
        bot.send_message(msg.chat.id, "You are not authorized to use admin commands.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let target_id: Option<i64> = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok());

    match target_id {
        Some(tid) => {
            if let Err(e) = analytics.set_user_blocked(tid, true) {
                log::error!("Failed to block user: {}", e);
                bot.send_message(msg.chat.id, "Failed to block user.")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, format!("User {} has been blocked.", tid))
                    .await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Usage: /block <user_id>")
                .await?;
        }
    }

    Ok(())
}

/// Handle the /unblock command - unblock a user.
pub async fn handle_unblock(
    bot: Bot,
    msg: Message,
    config: Config,
    analytics: Analytics,
) -> crate::HandlerResult {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !is_admin(user_id, &config) {
        bot.send_message(msg.chat.id, "You are not authorized to use admin commands.")
            .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let target_id: Option<i64> = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok());

    match target_id {
        Some(tid) => {
            if let Err(e) = analytics.set_user_blocked(tid, false) {
                log::error!("Failed to unblock user: {}", e);
                bot.send_message(msg.chat.id, "Failed to unblock user.")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, format!("User {} has been unblocked.", tid))
                    .await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Usage: /unblock <user_id>")
                .await?;
        }
    }

    Ok(())
}
