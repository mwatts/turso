//! Counts the SQL core compiles, by watching the `Preparing: {sql}` debug event
//! `Connection::prepare_with_origin` emits.
//!
//! ## Why this belongs in a test binary of its own
//!
//! `tracing` caches whether a callsite is worth evaluating **globally**, not per
//! thread. The first thread to reach the `Preparing:` event decides for every
//! thread, so if a test that does not install a subscriber compiles SQL first,
//! the event is pinned off and a recorder installed later sees nothing —
//! silently, as an empty recording rather than an error. `cargo test` runs the
//! tests in a binary on parallel threads, so any binary that mixes recording
//! tests with ordinary ones measures whichever ordering it happened to get.
//!
//! Installing the recorder as the process-wide subscriber, in a binary whose
//! only test is the one doing the measuring, removes the ordering entirely.

use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};

#[derive(Clone, Default)]
pub struct PreparedSql(Arc<Mutex<Vec<String>>>);

impl PreparedSql {
    /// Install as the process-wide subscriber. Call once, from a binary with a
    /// single test.
    pub fn install() -> Self {
        let recorded = Self::default();
        let subscriber = tracing_subscriber::registry().with(recorded.clone());
        tracing::subscriber::set_global_default(subscriber).expect("install the SQL recorder");
        recorded
    }

    /// Everything compiled since the last take.
    pub fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

struct FindPrepare(Option<String>);

impl Visit for FindPrepare {
    fn record_debug(&mut self, _field: &Field, value: &dyn std::fmt::Debug) {
        let text = format!("{value:?}");
        if let Some(sql) = text.strip_prefix("Preparing: ") {
            self.0 = Some(sql.to_owned());
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for PreparedSql {
    fn on_event(&self, event: &tracing::Event<'_>, _context: Context<'_, S>) {
        let mut found = FindPrepare(None);
        event.record(&mut found);
        if let Some(sql) = found.0 {
            self.0.lock().unwrap().push(sql);
        }
    }
}
