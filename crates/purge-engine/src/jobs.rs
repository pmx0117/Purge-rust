use crate::scanner::ScanEvent;
use purge_core::CleanupReport;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
};

pub type CancellableOutput<T> = (Sender<T>, Arc<AtomicBool>);
pub type DuplicateGroups = HashMap<u64, u64>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Scan,
    Cleanup,
    Duplicates,
}
#[derive(Debug)]
pub enum JobEvent {
    Scan(ScanEvent),
    Cleanup(CleanupReport),
    Duplicates(HashMap<u64, u64>),
    Interrupted(Operation),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Busy;
impl std::fmt::Display for Busy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("another operation is running")
    }
}
impl std::error::Error for Busy {}

enum ActiveJob {
    Scan {
        receiver: Receiver<ScanEvent>,
        cancel: Arc<AtomicBool>,
    },
    Cleanup(Receiver<CleanupReport>),
    Duplicates {
        receiver: Receiver<HashMap<u64, u64>>,
        cancel: Arc<AtomicBool>,
    },
}
#[derive(Default)]
pub struct Jobs {
    active: Option<ActiveJob>,
}
impl Jobs {
    pub fn operation(&self) -> Option<Operation> {
        self.active.as_ref().map(ActiveJob::operation)
    }
    pub fn busy(&self) -> bool {
        self.active.is_some()
    }
    pub fn can_close(&self) -> bool {
        self.operation() != Some(Operation::Cleanup)
    }
    fn ensure_idle(&self) -> Result<(), Busy> {
        if self.busy() { Err(Busy) } else { Ok(()) }
    }
    pub fn begin_scan(&mut self) -> Result<CancellableOutput<ScanEvent>, Busy> {
        self.begin_cancellable(ActiveJob::scan)
    }
    pub fn begin_cleanup(&mut self) -> Result<Sender<CleanupReport>, Busy> {
        self.ensure_idle()?;
        let (sender, receiver) = mpsc::channel();
        self.active = Some(ActiveJob::Cleanup(receiver));
        Ok(sender)
    }
    pub fn begin_duplicates(&mut self) -> Result<CancellableOutput<DuplicateGroups>, Busy> {
        self.begin_cancellable(ActiveJob::duplicates)
    }
    fn begin_cancellable<T>(
        &mut self,
        create: impl FnOnce(Receiver<T>, Arc<AtomicBool>) -> ActiveJob,
    ) -> Result<CancellableOutput<T>, Busy> {
        self.ensure_idle()?;
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        self.active = Some(create(receiver, cancel.clone()));
        Ok((sender, cancel))
    }
    pub fn cancellation_requested(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(ActiveJob::cancellation_requested)
    }
    pub fn cancel(&self) -> bool {
        self.active.as_ref().is_some_and(ActiveJob::cancel)
    }
    pub fn poll(&mut self) -> Vec<JobEvent> {
        let Some(active) = self.active.as_ref() else {
            return vec![];
        };
        let (events, finished) = active.poll();
        if finished {
            self.active = None;
        }
        events
    }
}
impl Drop for Jobs {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl ActiveJob {
    fn scan(receiver: Receiver<ScanEvent>, cancel: Arc<AtomicBool>) -> Self {
        Self::Scan { receiver, cancel }
    }
    fn duplicates(receiver: Receiver<DuplicateGroups>, cancel: Arc<AtomicBool>) -> Self {
        Self::Duplicates { receiver, cancel }
    }

    fn operation(&self) -> Operation {
        match self {
            Self::Scan { .. } => Operation::Scan,
            Self::Cleanup(_) => Operation::Cleanup,
            Self::Duplicates { .. } => Operation::Duplicates,
        }
    }
    fn cancellation_requested(&self) -> bool {
        match self {
            Self::Scan { cancel, .. } | Self::Duplicates { cancel, .. } => {
                cancel.load(Ordering::Relaxed)
            }
            Self::Cleanup(_) => false,
        }
    }
    fn cancel(&self) -> bool {
        match self {
            Self::Scan { cancel, .. } | Self::Duplicates { cancel, .. } => {
                cancel.store(true, Ordering::Relaxed);
                true
            }
            Self::Cleanup(_) => false,
        }
    }
    fn poll(&self) -> (Vec<JobEvent>, bool) {
        match self {
            Self::Scan { receiver, .. } => poll_scan(receiver),
            Self::Cleanup(receiver) => poll_result(receiver, Operation::Cleanup, JobEvent::Cleanup),
            Self::Duplicates { receiver, .. } => {
                poll_result(receiver, Operation::Duplicates, JobEvent::Duplicates)
            }
        }
    }
}
fn poll_scan(receiver: &Receiver<ScanEvent>) -> (Vec<JobEvent>, bool) {
    let mut events = vec![];
    for _ in 0..300 {
        match receiver.try_recv() {
            Ok(event) => {
                let finished = matches!(event, ScanEvent::Finished { .. });
                events.push(JobEvent::Scan(event));
                if finished {
                    return (events, true);
                }
            }
            Err(mpsc::TryRecvError::Empty) => return (events, false),
            Err(mpsc::TryRecvError::Disconnected) => {
                events.push(JobEvent::Interrupted(Operation::Scan));
                return (events, true);
            }
        }
    }
    (events, false)
}
fn poll_result<T>(
    receiver: &Receiver<T>,
    operation: Operation,
    event: impl FnOnce(T) -> JobEvent,
) -> (Vec<JobEvent>, bool) {
    match receiver.try_recv() {
        Ok(value) => (vec![event(value)], true),
        Err(mpsc::TryRecvError::Empty) => (vec![], false),
        Err(mpsc::TryRecvError::Disconnected) => (vec![JobEvent::Interrupted(operation)], true),
    }
}
