use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub fn print_value(value: u32) {
    println!("{value}");
}

pub fn print_error(value: u32) {
    eprintln!("{value}");
}

pub fn write_stdout(bytes: &[u8]) -> std::io::Result<()> {
    std::io::stdout().write_all(bytes)
}

pub fn locked(counter: &Mutex<u32>) -> u32 {
    *counter.lock().expect("unpoisoned counter")
}

pub fn atomic(counter: &AtomicU32) -> u32 {
    counter.fetch_add(1, Ordering::SeqCst)
}

pub fn channel() -> u32 {
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send(1).expect("open channel");
    receiver.recv().expect("sent value")
}

pub fn shared(value: u32) -> Arc<u32> {
    Arc::new(value)
}

pub fn formatted(value: u32) -> String {
    format!("{value}")
}
