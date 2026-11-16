# Local follow-ups (development fork)

This fork uses existing local labels for a manual **Pending** flag. Create a
label named “Pending”, apply it from a chat's context menu, and use the label
filter to see those chats. Labels stay in the local archive and do not sync to
WhatsApp.

The chat menu also has **Private note and reminder…**. It stores a note of up
to 500 characters and an optional reminder in the local SQLite archive. The
reminder can be set for one hour, one day, or three days from now. A bell in
the chat row turns to the accent colour when due. Remove it in the same dialog.

The reminder is an in-app visual cue: this version does not issue an operating
system notification, and the app must be open to see it. Notes and reminders
are removed when the chat is deleted or the account is unlinked. Clearing the
messages of a chat keeps its note and reminder.

No message content is read or exported by this feature. No new WhatsApp
protocol action is sent when a note or reminder changes.
