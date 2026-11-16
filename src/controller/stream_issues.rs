//! Audio capture failures this provider reports first-hand.
//!
//! A capture stream remembers the operation ID its open request carried
//! (stream-session names it `ss-<id>-audio…`). Failures are queued here while
//! the controller lock is held, by request handling or by a native read; the
//! module tick takes the queue and publishes it on the stream-report topic.
use amaterasu_module_sdk::stream_lifecycle::{LifecycleLayer, StreamIssue, StreamIssueOutbox};

/// Reporting state one capture stream carries.
#[derive(Default)]
pub(super) struct StreamIssueState {
    operation: Option<String>,
    failure_reported: bool,
    recovery_reported: bool,
}

impl StreamIssueState {
    pub(super) fn new(operation: Option<String>) -> Self {
        Self {
            operation,
            ..Self::default()
        }
    }
}

/// Queues this controller's stream issues. Off by default so controller
/// tests see no reports; the module entry enables it.
#[derive(Default)]
pub(super) struct StreamIssueReporter {
    enabled: bool,
    outbox: StreamIssueOutbox,
}

impl StreamIssueReporter {
    pub(super) fn enable(&mut self) {
        self.enabled = true;
    }

    /// The capture for an open request could not be started.
    #[track_caller]
    pub(super) fn open_failed(&mut self, operation: Option<&str>, code: &'static str) {
        if let Some(operation) = operation {
            self.push(StreamIssue::critical(
                operation,
                LifecycleLayer::Audio,
                "open_capture",
                code,
            ));
        }
    }

    /// An open capture stream stopped delivering PCM for good. Reported once.
    #[track_caller]
    pub(super) fn capture_failed(&mut self, state: &mut StreamIssueState, code: &'static str) {
        if state.failure_reported {
            return;
        }
        state.failure_reported = true;
        if let Some(operation) = &state.operation {
            self.push(StreamIssue::critical(
                operation.clone(),
                LifecycleLayer::Audio,
                "read_capture",
                code,
            ));
        }
    }

    /// The capture device went away and the worker is reopening it. Reported
    /// once per interruption.
    #[track_caller]
    pub(super) fn observe_recovery(&mut self, state: &mut StreamIssueState, recovering: bool) {
        if !recovering {
            state.recovery_reported = false;
            return;
        }
        if state.recovery_reported {
            return;
        }
        state.recovery_reported = true;
        if let Some(operation) = &state.operation {
            self.push(StreamIssue::demoted(
                operation.clone(),
                LifecycleLayer::Audio,
                "capture_recovery",
                "audio_capture_interrupted",
                "audio_capture_reopen",
            ));
        }
    }

    /// Hand every queued report to the module tick.
    pub(super) fn take(&mut self) -> StreamIssueOutbox {
        std::mem::take(&mut self.outbox)
    }

    fn push(&mut self, issue: StreamIssue) {
        if !self.enabled {
            return;
        }
        if issue.valid() {
            self.outbox.push(issue);
        } else {
            eprintln!(
                "amaterasu audio stream issue invalid phase={} code={}",
                issue.phase, issue.code
            );
        }
    }
}
