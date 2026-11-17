//! Private follow-ups keyed to individual messages, never sent to WhatsApp.

use rusqlite::params;

use super::{Archive, Result};
use crate::model::{FollowupSummary, LocalFollowup};

pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS local_message_followups (
    chat TEXT NOT NULL,
    id TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    remind_at INTEGER,
    done INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (chat, id)
);
CREATE INDEX IF NOT EXISTS local_followups_pending ON local_message_followups (chat, done, remind_at);
CREATE TRIGGER IF NOT EXISTS delete_local_message_followup AFTER DELETE ON messages BEGIN
    DELETE FROM local_message_followups WHERE chat = OLD.chat AND id = OLD.id;
END;
";

const NOTE_LIMIT: usize = 500;

impl Archive {
    pub fn local_followups(&self, chat: &str) -> Result<Vec<(String, LocalFollowup)>> {
        let mut query = self.connection.prepare(
            "SELECT id, note, remind_at, done FROM local_message_followups WHERE chat = ?1",
        )?;
        query
            .query_map([chat], |row| {
                Ok((
                    row.get(0)?,
                    LocalFollowup {
                        note: row.get(1)?,
                        remind_at: row.get(2)?,
                        done: row.get(3)?,
                    },
                ))
            })?
            .collect()
    }

    pub fn local_followup_summary(&self, chat: &str) -> Result<FollowupSummary> {
        self.connection.query_row(
            "SELECT COUNT(*), MIN(remind_at) FROM local_message_followups
             WHERE chat = ?1 AND done = 0",
            [chat],
            |row| {
                Ok(FollowupSummary {
                    pending: row.get(0)?,
                    next_due: row.get(1)?,
                })
            },
        )
    }

    pub fn set_local_followup(
        &self,
        chat: &str,
        id: &str,
        note: &str,
        remind_at: Option<i64>,
        done: bool,
        remove: bool,
    ) -> Result<()> {
        let note: String = note.trim().chars().take(NOTE_LIMIT).collect();
        if remove {
            self.connection.execute(
                "DELETE FROM local_message_followups WHERE chat = ?1 AND id = ?2",
                params![chat, id],
            )?;
        } else {
            self.connection.execute(
                "INSERT INTO local_message_followups (chat, id, note, remind_at, done)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(chat, id) DO UPDATE SET note = excluded.note,
                 remind_at = excluded.remind_at, done = excluded.done",
                params![chat, id, note, remind_at, done],
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_requests_in_one_group() {
        let archive = Archive::in_memory().unwrap();
        archive
            .set_local_followup("group", "blank", "", None, false, false)
            .unwrap();
        assert_eq!(archive.local_followup_summary("group").unwrap().pending, 1);
        archive
            .set_local_followup("group", "a", "First", Some(100), false, false)
            .unwrap();
        archive
            .set_local_followup("group", "b", "Second", Some(200), false, false)
            .unwrap();
        assert_eq!(archive.local_followups("group").unwrap().len(), 3);
        assert_eq!(archive.local_followup_summary("group").unwrap().pending, 3);
        archive
            .set_local_followup("group", "a", "First", Some(100), true, false)
            .unwrap();
        let summary = archive.local_followup_summary("group").unwrap();
        assert_eq!(summary.pending, 2);
        assert_eq!(summary.next_due, Some(200));
        archive
            .set_local_followup("group", "b", "", None, false, true)
            .unwrap();
        assert_eq!(archive.local_followups("group").unwrap().len(), 2);
        archive
            .set_local_followup("other", "a", "Different chat", None, false, false)
            .unwrap();
        assert_eq!(archive.local_followups("other").unwrap().len(), 1);
        assert!(
            archive
                .local_followups("group")
                .unwrap()
                .iter()
                .any(|(_, request)| request.note == "First")
        );
    }

    #[test]
    fn deleting_a_message_clears_its_request() {
        let archive = Archive::in_memory().unwrap();
        archive
            .connection
            .execute(
                "INSERT INTO messages (chat, id, sender, from_me, timestamp, content)
             VALUES ('group', 'm1', 'sender', 0, 1, '{}')",
                [],
            )
            .unwrap();
        archive
            .set_local_followup("group", "m1", "Call back", Some(123), false, false)
            .unwrap();
        archive
            .connection
            .execute(
                "DELETE FROM messages WHERE chat = 'group' AND id = 'm1'",
                [],
            )
            .unwrap();
        assert!(archive.local_followups("group").unwrap().is_empty());
    }
}
