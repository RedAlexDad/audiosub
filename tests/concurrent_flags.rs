use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// Spawn a worker thread that loops, checking atomic stop/pause flags.
/// Mirrors the real worker pattern (Arc<AtomicBool> + mpsc).
struct WorkerHandle {
    stop: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    join: thread::JoinHandle<()>,
}

impl WorkerHandle {
    fn spawn(tx: mpsc::Sender<&'static str>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let pause_clone = Arc::clone(&pause);

        let join = thread::spawn(move || {
            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    tx.send("stopped").ok();
                    break;
                }
                if pause_clone.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
                thread::sleep(Duration::from_millis(5));
            }
        });

        Self { stop, pause, join }
    }

    fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    fn pause(&self) {
        self.pause.store(true, Ordering::Relaxed);
    }

    fn resume(&self) {
        self.pause.store(false, Ordering::Relaxed);
    }
}

#[test]
fn stop_flag_terminates_worker() {
    let (tx, rx) = mpsc::channel();
    let worker = WorkerHandle::spawn(tx);
    let start = Instant::now();
    worker.stop();
    worker.join.join().unwrap();
    assert!(start.elapsed() < Duration::from_secs(5));
    assert_eq!(rx.recv().unwrap(), "stopped");
}

#[test]
fn pause_and_resume_does_not_terminate() {
    let (tx, rx) = mpsc::channel();
    let worker = WorkerHandle::spawn(tx);
    worker.pause();
    thread::sleep(Duration::from_millis(30));
    worker.resume();
    thread::sleep(Duration::from_millis(30));
    worker.stop();
    worker.join.join().unwrap();
    assert_eq!(rx.recv().unwrap(), "stopped");
}

#[test]
fn multiple_workers_all_respond_to_stop() {
    let count = 8;
    let mut workers = Vec::new();
    let mut receivers = Vec::new();
    for _ in 0..count {
        let (tx, rx) = mpsc::channel();
        workers.push(WorkerHandle::spawn(tx));
        receivers.push(rx);
    }
    thread::sleep(Duration::from_millis(10));
    for w in &workers {
        w.stop();
    }
    for w in workers {
        w.join.join().unwrap();
    }
    for rx in &receivers {
        assert_eq!(rx.recv().unwrap(), "stopped");
    }
}

#[test]
fn atomic_bool_reset_works() {
    let flag = Arc::new(AtomicBool::new(true));
    assert!(flag.load(Ordering::Relaxed));
    flag.store(false, Ordering::Relaxed);
    assert!(!flag.load(Ordering::Relaxed));
    flag.store(true, Ordering::Relaxed);
    assert!(flag.load(Ordering::Relaxed));
}

#[test]
fn channel_message_passing() {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let msg = rx.recv().unwrap();
        assert_eq!(msg, "hello");
        "ok"
    });
    tx.send("hello").unwrap();
    let result = handle.join().unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn atomic_bool_from_multiple_threads() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let f = Arc::clone(&flag);
        handles.push(thread::spawn(move || {
            f.store(true, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert!(flag.load(Ordering::SeqCst));
}
