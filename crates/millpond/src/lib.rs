//! millpond: the inbox for events outside a repository that should change
//! its work (boomdrivedev/blueprints `design/millpond/requirements.md`).
//!
//! It takes each offer once by key into an inbox on a shared branch, by
//! weir's capture contract, records and write protocol (C.1, C.2); banks
//! raw material on the host that took it and keeps each adapter's cursor
//! and status there (C.3); reads the keeper's policy as each capture lands,
//! gathering what a rule matches into its item (A.14); and, on the host the
//! policy names, closes each rule's item at its window and sends it to a
//! place of the rule's kind through `flume launch`, reading sluice's record
//! of the place's branch at each tick (A.9, C.5, C.7). Every rule of the
//! capture contract, the records and the policy is weir's; millpond keeps
//! none of its own (section 9).

pub mod bank;
pub mod clock;
pub mod host;
pub mod items;
pub mod run;
pub mod send;
pub mod settings;
pub mod state;
pub mod store;
pub mod take;
pub mod tick;
