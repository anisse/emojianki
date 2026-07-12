use log::{Level, Metadata, Record};
use std::error::Error;
struct TestLogger;

use std::sync::Once;
static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| init_log().unwrap());
}

fn init_log() -> Result<(), Box<dyn Error>> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(Level::Debug.to_level_filter());
    Ok(())
}

impl log::Log for TestLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("{}", record.args());
    }

    fn flush(&self) {}
}

static LOGGER: TestLogger = TestLogger;
