use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use chrono::Local;
use global_placeholders::global;

pub struct Logger {
  file: File,
}

impl Logger {
  pub fn new() -> io::Result<Self> {
    let file = OpenOptions::new().create(true).append(true).open(global!("omnitron.log"))?;
    Ok(Logger { file })
  }

  pub fn write(&mut self, message: &str) {
    log::info!("{message}");
    writeln!(
      &mut self.file,
      "[{}] {}",
      Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
      message
    )
    .unwrap()
  }
}

#[macro_export]
macro_rules! log {($($arg:tt)*) =>
    { log::Logger::new().unwrap().write(format!($($arg)*).as_str()) }}
