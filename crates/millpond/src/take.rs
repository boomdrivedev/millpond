//! Taking lines: weir's seven steps (C.1), through an inbox that writes
//! each capture in one write of the shared branch (104) and the host's
//! bank, with the keeper's policy read in each capture's write where it
//! stands (127, 128); and the import, one write of weir's import (60 to
//! 63, 119, 120).

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use weir_branch::Branch;
use weir_capture::{
    read_import_line, take, CameIn, Inbox as TakeInbox, Key, Refusal, Refused, Report, Take, Taken,
};
use weir_signals::ImportedCapture;

use crate::bank::FolderBank;
use crate::clock;
use crate::host::Inbox;
use crate::settings::Settings;
use crate::store;

/// The inbox take writes through: each capture one write of the branch,
/// formed and checked at its tip (9, 104; C.2).
struct BranchInbox<'a> {
    branch: &'a Branch,
    held: BTreeSet<Key>,
    who: &'a str,
    stale_after: Duration,
}

impl TakeInbox for BranchInbox<'_> {
    fn holds(&self, key: &Key) -> bool {
        self.held.contains(key)
    }

    fn write(&mut self, taken: &Taken) -> Result<(), Refusal> {
        let what = format!("take {}", taken.capture.key);
        store::write(self.branch, &what, self.who, self.stale_after, |inbox| {
            inbox
                .take(taken)
                .map(Some)
                .map_err(|refused| refused.to_string())
        })
        .map(|_| ())
        .map_err(Refusal)
    }
}

/// What a take needs of its host.
pub struct Taker<'a> {
    pub inbox: &'a Inbox,
    pub machine: &'a str,
    /// The limit, the settings' unless a command named one.
    pub limit: usize,
}

impl Taker<'_> {
    /// Takes `lines` through the branch's inbox and this host's bank, the
    /// keys the tip holds before the take held, the policy of the settings
    /// at that tip read in each capture's write where it stands (C.1).
    pub fn take_lines(
        &self,
        lines: &[String],
        raw_dir: Option<&Path>,
        came_in: CameIn,
    ) -> Result<Report, String> {
        let (_, inbox, settings) = store::read(&self.inbox.branch)?;
        let held = inbox.keys().cloned().collect();
        let who = store::who(self.machine);
        let mut writer = BranchInbox {
            branch: &self.inbox.branch,
            held,
            who: &who,
            stale_after: settings.lock_stale,
        };
        let mut bank = FolderBank {
            root: self.inbox.bank_dir(),
        };
        let how = take_with(&settings, self.machine, self.limit, came_in)?;
        Ok(take(lines, raw_dir, &how, &mut writer, &mut bank))
    }

    /// The import of `lines` as one write, formed and checked at the tip:
    /// a line that is no import line refused by its number, and each
    /// capture the import refuses by its line (60 to 63, 119, 120).
    pub fn import(&self, lines: &[String]) -> Result<Report, String> {
        let mut refused = Vec::new();
        let mut captures: Vec<ImportedCapture> = Vec::new();
        let mut line_of = Vec::new();
        for (n, line) in (1..).zip(lines) {
            match read_import_line(line) {
                Ok(read) => {
                    captures.push(read.into());
                    line_of.push(n);
                }
                Err(malformed) => refused.push(Refused {
                    line: n,
                    key: malformed.key.clone(),
                    reason: malformed.to_string(),
                }),
            }
        }
        let (_, _, settings) = store::read(&self.inbox.branch)?;
        let now = clock::offset();
        let mut imported = None;
        store::write(
            &self.inbox.branch,
            "import",
            &store::who(self.machine),
            settings.lock_stale,
            |inbox| {
                let done = inbox
                    .import(&captures, self.machine, now)
                    .map_err(|refused| refused.to_string())?;
                imported = Some(done.report);
                Ok(done.write)
            },
        )?;
        let report = imported.ok_or("the import formed no write")?;
        for refusal in report.refused {
            let line = captures
                .iter()
                .position(|capture| capture.key == refusal.key)
                .map_or(0, |at| line_of[at]);
            refused.push(Refused {
                line,
                key: Some(refusal.key),
                reason: format!("`{}` {}", refusal.field, refusal.why),
            });
        }
        refused.sort_by_key(|refusal| refusal.line);
        Ok(Report {
            taken: report.written,
            held: report.held,
            refused,
            ..Report::default()
        }
        .finish(true))
    }
}

/// The take of a host, given the keeper's policy where it stands (125).
fn take_with(
    settings: &Settings,
    machine: &str,
    limit: usize,
    came_in: CameIn,
) -> Result<Take, String> {
    let take = Take::new(machine, clock::offset(), limit, came_in).map_err(|e| e.to_string())?;
    Ok(match &settings.keeper {
        Ok(keeper) if !keeper.policy.rules().is_empty() => {
            take.with_policy(keeper.policy.clone(), keeper.people.clone(), clock::zone())
        }
        _ => take,
    })
}
