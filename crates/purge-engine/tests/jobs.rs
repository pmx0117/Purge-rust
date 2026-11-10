use purge_core::CleanupReport;
use purge_engine::{
    jobs::{JobEvent, Jobs, Operation},
    scanner::ScanEvent,
};
use std::sync::atomic::Ordering;

#[test]
fn a_disconnected_worker_releases_the_operation_and_reports_the_failure() {
    let mut jobs = Jobs::default();
    let (sender, _) = jobs.begin_scan().unwrap();
    drop(sender);
    let events = jobs.poll();
    assert!(
        !jobs.busy(),
        "a stopped worker must not leave the application permanently busy"
    );
    assert!(matches!(
        events.as_slice(),
        [JobEvent::Interrupted(Operation::Scan)]
    ));
    assert!(jobs.begin_cleanup().is_ok());
}

#[test]
fn only_one_operation_can_own_the_application_at_a_time() {
    let mut jobs = Jobs::default();
    let (_sender, cancel) = jobs.begin_scan().unwrap();
    assert!(jobs.begin_cleanup().is_err());
    assert!(jobs.begin_duplicates().is_err());
    assert_eq!(jobs.operation(), Some(Operation::Scan));
    assert!(jobs.cancel());
    assert!(cancel.load(Ordering::Relaxed));
    assert!(jobs.can_close());
}

#[test]
fn a_cleanup_cannot_be_cancelled_or_closed_until_its_report_arrives() {
    let mut jobs = Jobs::default();
    let sender = jobs.begin_cleanup().unwrap();
    assert!(!jobs.can_close());
    assert!(!jobs.cancel());
    assert!(jobs.poll().is_empty());
    sender
        .send(CleanupReport {
            bytes: 100,
            moved_ids: vec![7],
            ..Default::default()
        })
        .unwrap();
    let events = jobs.poll();
    assert!(
        matches!(&events[0],JobEvent::Cleanup(report)if report.bytes==100&&report.moved_ids==[7])
    );
    assert!(!jobs.busy());
    assert!(jobs.can_close());
}

#[test]
fn scan_results_stream_in_bounded_batches_and_finish_exactly_once() {
    let mut jobs = Jobs::default();
    let (sender, _) = jobs.begin_scan().unwrap();
    for _ in 0..310 {
        sender.send(ScanEvent::Progress("checking".into())).unwrap();
    }
    sender
        .send(ScanEvent::Finished { cancelled: true })
        .unwrap();
    assert_eq!(jobs.poll().len(), 300);
    assert!(jobs.busy());
    let remaining = jobs.poll();
    assert_eq!(remaining.len(), 11);
    assert!(matches!(
        remaining.last(),
        Some(JobEvent::Scan(ScanEvent::Finished { cancelled: true }))
    ));
    assert!(!jobs.busy());
    assert!(jobs.poll().is_empty());
}

#[test]
fn duplicate_check_returns_the_groups_then_becomes_idle() {
    let mut jobs = Jobs::default();
    let (sender, cancel) = jobs.begin_duplicates().unwrap();
    assert_eq!(jobs.operation(), Some(Operation::Duplicates));
    assert!(!cancel.load(Ordering::Relaxed));
    sender.send([(1, 1), (2, 1)].into()).unwrap();
    let events = jobs.poll();
    assert!(matches!(&events[0],JobEvent::Duplicates(groups)if groups.len()==2));
    assert!(!jobs.busy());
}

#[test]
fn other_worker_disconnects_are_visible_and_never_keep_the_application_locked() {
    let mut jobs = Jobs::default();
    let sender = jobs.begin_cleanup().unwrap();
    drop(sender);
    assert!(matches!(
        jobs.poll().as_slice(),
        [JobEvent::Interrupted(Operation::Cleanup)]
    ));
    assert!(jobs.can_close());
    let (sender, _) = jobs.begin_duplicates().unwrap();
    assert!(jobs.poll().is_empty());
    drop(sender);
    assert!(matches!(
        jobs.poll().as_slice(),
        [JobEvent::Interrupted(Operation::Duplicates)]
    ));
    assert!(!jobs.busy());
}

#[test]
fn dropping_a_scanner_owner_requests_cooperative_cancellation() {
    let mut jobs = Jobs::default();
    let (_sender, cancel) = jobs.begin_scan().unwrap();
    drop(jobs);
    assert!(cancel.load(Ordering::Relaxed));
}
