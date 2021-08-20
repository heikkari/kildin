use std::fmt::{Formatter, Display as FmtDisplay, Result as FmtResult};
#[derive(Debug, Clone)]
pub enum Level {
    Info,
    Warn,
    Error,
    Fatal,
}

impl FmtDisplay for Level {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct Logger {}

impl Logger {
    pub fn new() -> Self { Self {} }

    pub fn log(&self, level: Level, msg: &str) {
        println!("[{:?}] {}", level, msg);
    }
}