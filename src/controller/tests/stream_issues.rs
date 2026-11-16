//! First-hand audio capture reports under the opening operation ID.
use super::*;
use amaterasu_module_sdk::stream_lifecycle::{
    IssueDisposition, LifecycleLayer, StreamIssue, StreamReport,
};

fn published(controller: &mut Controller) -> Vec<(Value, StreamIssue)> {
    let mut commands = Vec::new();
    controller
        .take_stream_issues()
        .flush_with(&mut |command| {
            commands.push(command);
            Ok(Response::Ok)
        })
        .unwrap();
    commands
        .into_iter()
        .filter_map(|command| match command {
            Command::Publish { topic, payload } if topic == "amaterasu.stream.report/1" => {
                match serde_json::from_value(payload.clone()).unwrap() {
                    StreamReport::Issue(issue) => Some((payload, issue)),
                    StreamReport::Binding(_) => None,
                }
            }
            _ => None,
        })
        .collect()
}

fn open_endpoint(endpoint: &str) -> Operation {
    let mut operation = open();
    let Operation::Open {
        endpoint: requested,
        operation_id,
        ..
    } = &mut operation
    else {
        unreachable!();
    };
    *requested = endpoint.into();
    *operation_id = Some("ss-2a-audio".into());
    operation
}

#[test]
fn a_failed_capture_open_is_published_under_the_opening_operation() {
    let (mut controller, _) = controller(true);
    controller.enable_stream_issue_reports();
    assert_eq!(
        controller
            .handle("alice", open_endpoint("render.missing"))
            .err(),
        Some("endpoint_not_found")
    );

    let published = published(&mut controller);
    assert_eq!(published.len(), 1);
    let (payload, issue) = &published[0];
    assert_eq!(payload["kind"], "issue");
    assert_eq!(issue.operation, "ss-2a-audio");
    assert_eq!(issue.layer, LifecycleLayer::Audio);
    assert_eq!(issue.phase, "open_capture");
    assert_eq!(issue.code, "endpoint_not_found");
    assert_eq!(issue.disposition, IssueDisposition::CriticalFailure);
    assert!(issue.caused_by.is_none());
    assert!(issue.site.contains("controller.rs:"), "{}", issue.site);
}

#[test]
fn capture_recovery_and_terminal_failure_are_each_reported_once() {
    let (mut controller, observed) = controller(true);
    controller.enable_stream_issue_reports();
    let stream = controller
        .handle("alice", open_endpoint("render.explicit"))
        .unwrap()
        .created
        .unwrap();

    observed.health.lock().unwrap().recovering = true;
    let recovery = published(&mut controller);
    assert!(
        published(&mut controller).is_empty(),
        "one per interruption"
    );
    assert_eq!(recovery.len(), 1);
    assert_eq!(recovery[0].1.code, "audio_capture_interrupted");
    assert_eq!(recovery[0].1.disposition, IssueDisposition::DemotedWarning);
    assert_eq!(
        recovery[0].1.fallback.as_deref(),
        Some("audio_capture_reopen")
    );

    *observed.health.lock().unwrap() = Health {
        failure: Some("audio_capture_failed"),
        ..Health::default()
    };
    let mut sink = Sink {
        reject: false,
        packets: Vec::new(),
    };
    let callback = AudioSink {
        context: (&mut sink as *mut Sink).cast(),
        write,
    };
    // SAFETY: the sink context outlives these synchronous reads.
    unsafe {
        assert_eq!(controller.read("alice", stream, callback), STATUS_OK);
        assert_eq!(
            controller.read("alice", stream, callback),
            MEDIA_BACKEND_ERROR
        );
        assert_eq!(
            controller.read("alice", stream, callback),
            MEDIA_BACKEND_ERROR
        );
    }
    let failure = published(&mut controller);
    assert_eq!(failure.len(), 1, "{failure:?}");
    let issue = &failure[0].1;
    assert_eq!(issue.operation, "ss-2a-audio");
    assert_eq!(issue.phase, "read_capture");
    assert_eq!(issue.code, "audio_capture_failed");
    assert_eq!(issue.disposition, IssueDisposition::CriticalFailure);
    assert!(issue.site.contains("controller.rs:"), "{}", issue.site);
}

#[test]
fn reports_are_off_until_enabled() {
    let (mut controller, _) = controller(true);
    let _ = controller.handle("alice", open_endpoint("render.missing"));
    assert!(published(&mut controller).is_empty());
}
