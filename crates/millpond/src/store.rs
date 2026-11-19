//! The inbox store: the shared branch, read and written by weir's
//! protocol, every write formed from the inbox read at the tip and checked
//! there by weir's records (4, 80 to 82; C.2).

use std::time::Duration;

use time::OffsetDateTime;
use weir_branch::{Ask, Branch, Files, Tip, Written};
use weir_signals::{InboxAt, Write};

use crate::settings::{self, Settings};

/// The inbox at `tip`, read whole.
pub fn inbox_at(tip: Option<&Tip>) -> Result<InboxAt, String> {
    let files = match tip {
        Some(tip) => tip.files("inbox/").map_err(|error| error.to_string())?,
        None => Files::new(),
    };
    InboxAt::read(&files).map_err(|error| error.to_string())
}

/// The settings at `tip`, read at the version the inbox was (C.2).
pub fn settings_at(tip: Option<&Tip>) -> Result<Settings, String> {
    let text = match tip {
        Some(tip) => tip
            .file(settings::PATH)
            .map_err(|error| error.to_string())?,
        None => None,
    };
    Settings::read(text.as_deref())
}

/// The branch read once: its tip, the inbox and the settings there.
pub fn read(branch: &Branch) -> Result<(Option<Tip>, InboxAt, Settings), String> {
    let tip = weir_branch::read(branch).map_err(|failure| failure.to_string())?;
    let inbox = inbox_at(tip.as_ref())?;
    let settings = settings_at(tip.as_ref())?;
    Ok((tip, inbox, settings))
}

/// Writes the branch: `form` gives the write from the inbox at the tip
/// read, again at each new tip the branch moved to, or none for nothing to
/// write, or refuses naming why (81).
pub fn write(
    branch: &Branch,
    what: &str,
    who: &str,
    stale_after: Duration,
    mut form: impl FnMut(&InboxAt) -> Result<Option<Write>, String>,
) -> Result<Written, String> {
    let ask = Ask {
        what,
        who,
        bound: None,
        stale_after,
        clock: &OffsetDateTime::now_utc,
    };
    weir_branch::write(branch, &ask, |tip| {
        let inbox = inbox_at(tip)?;
        Ok(form(&inbox)?.map(|write| write.files).unwrap_or_default())
    })
    .map_err(|error| error.to_string())
}

/// Who asks for every write a host makes (50).
pub fn who(machine: &str) -> String {
    format!("millpond@{machine}")
}
