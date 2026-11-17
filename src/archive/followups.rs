//! Private follow-ups. These rows are never sent to WhatsApp or another device.

use rusqlite::{OptionalExtension, params};

use super::{Archive, Result};
use crate::model::LocalFollowup;

pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS local_chat_followups (
    chat TEXT PRIMARY KEY,
    note TEXT NOT NULL DEFAULT '',
    remind_at INTEGER
);
";

/// A short working note, rather than a second message archive.
const NOTE_LIMIT: usize = 500;

impl Archive {
    pub fn local_followup(&self, chat: &str) -> Result<LocalFollowup> {
        Ok(self
            .connection
            .query_row(
                "SELECT note, remind_at FROM local_chat_followups WHERE chat = ?1",
                [chat],
                |row| {
                    Ok(LocalFollowup {
                        note: row.get(0)?,
                        remind_at: row.get(1)?,
                    })
                },
            )
            .optional()?
            .unwrap_or_default())
    }

    pub fn set_local_followup(&self, chat: &str, note: &str, remind_at: Option<i64>) -> Result<()> {
        let note: String = note.trim().chars().take(NOTE_LIMIT).collect();
        if note.is_empty() && remind_at.is_none() {
            self.connection
                .execute("DELETE FROM local_chat_followups WHERE chat = ?1", [chat])?;
        } else {
            self.connection.execute(
                "INSERT INTO local_chat_followups (chat, note, remind_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(chat) DO UPDATE SET note = excluded.note, remind_at = excluded.remind_at",
                params![chat, note, remind_at],
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn followup_round_trip_and_clear() {
        let archive = Archive::in_memory().unwrap();
        assert_eq!(
            archive.local_followup("chat").unwrap(),
            LocalFollowup::default()
        );
        archive
            .set_local_followup("chat", "  Call back  ", Some(123))
            .unwrap();
        assert_eq!(
            archive.local_followup("chat").unwrap(),
            LocalFollowup {
                note: "Call back".into(),
                remind_at: Some(123),
            }
        );
        archive.set_local_followup("chat", "", None).unwrap();
        assert_eq!(
            archive.local_followup("chat").unwrap(),
            LocalFollowup::default()
        );
    }
}
