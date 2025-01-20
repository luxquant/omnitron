#[macro_use]
mod log;
mod api;
mod fork;

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use api::{DAEMON_CPU_PERCENTAGE, DAEMON_MEM_USAGE, DAEMON_START_TIME};
use chrono::Utc;
use fork::{daemon, Fork};
use global_placeholders::global;
use macros_rs::{crashln, str, then};
use psutil::process::Process;

use crate::config;
use crate::helpers::{self};
use crate::process::id::Id;
use crate::process::{hash, Runner, Status};
static ENABLE_API: AtomicBool = AtomicBool::new(false);
static ENABLE_WEBUI: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_termination_signal(_: libc::c_int) {
  pid::remove();
  // daemon_log!("[daemon] killed", "pid" => process::id());
  unsafe { libc::_exit(0) }
}

fn restart_process() {
  for (id, item) in Runner::new().items_mut() {
    let mut runner = Runner::new();
    let children = crate::service::find_chidren(item.pid);

    if !children.is_empty() && children != item.children {
      // daemon_log!("[daemon] added", "children" => format!("{children:?}"));
      runner.set_children(*id, children).save();
    }

    if item.running && item.watch.enabled {
      let path = item.path.join(item.watch.path.clone());
      let hash = hash::create(path);

      if hash != item.watch.hash {
        runner.restart(item.id, false);
        // daemon_log!("[daemon] watch reload", "name" => item.name, "hash" => "hash");
        continue;
      }
    }

    if !item.running && pid::running(item.pid as i32) {
      Runner::new().set_status(*id, Status::Running);
      // daemon_log!("[daemon] process fix status", "name" => item.name, "id" => id);
      continue;
    }

    then!(!item.running || pid::running(item.pid as i32), continue);

    if item.running && item.crash.value == config::read().daemon.restarts {
      // daemon_log!("[daemon] process has crashed", "name" => item.name, "id" => id);
      runner.stop(item.id);
      runner.set_crashed(*id).save();
      continue;
    } else {
      runner.get(item.id).crashed();
      // daemon_log!("[daemon] restarted", "name" => item.name, "id" => id, "crashes" => item.crash.value);
    }
  }
}

pub fn stop() {
  if pid::exists() {
    println!("{} Stopping OMNITRON daemon", *helpers::SUCCESS);

    match pid::read() {
      Ok(pid) => {
        crate::service::stop(pid.get());
        pid::remove();
        // daemon_log!("[daemon] stopped", "pid" => pid);
        println!("{} OMNITRON daemon stopped", *helpers::SUCCESS);
      }
      Err(err) => crashln!("{} Failed to read PID file: {}", *helpers::FAIL, err),
    }
  } else {
    crashln!("{} The daemon is not running", *helpers::FAIL)
  }
}

pub fn start(verbose: bool) {
  let external = match global!("omnitron.daemon.kind").as_str() {
    "external" => true,
    "default" => false,
    "rust" => false,
    "cc" => true,
    _ => false,
  };

  println!(
    "{} Spawning OMNITRON daemon (omnitron_base={})",
    *helpers::SUCCESS,
    global!("omnitron.base")
  );

  if ENABLE_API.load(Ordering::Acquire) {
    println!(
      "{} API server started (address={}, webui={})",
      *helpers::SUCCESS,
      config::read().fmt_address(),
      ENABLE_WEBUI.load(Ordering::Acquire)
    );
  }

  if pid::exists() {
    match pid::read() {
      Ok(pid) => then!(!pid::running(pid.get()), pid::remove()),
      Err(_) => crashln!("{} The daemon is already running", *helpers::FAIL),
    }
  }

  #[inline]
  #[tokio::main]
  async extern "C" fn init() {
    pid::name("OMNITRON Restart Handler Daemon");

    let config = config::read().daemon;
    let api_enabled = ENABLE_API.load(Ordering::Acquire);
    let ui_enabled = ENABLE_WEBUI.load(Ordering::Acquire);

    unsafe { libc::signal(libc::SIGTERM, handle_termination_signal as usize) };
    DAEMON_START_TIME.set(Utc::now().timestamp_millis() as f64);

    pid::write(process::id());
    // daemon_log!("[daemon] new fork", "pid" => process::id());

    if api_enabled {
      // daemon_log!("[api] server queued", "address" => config::read().fmt_address());
      tokio::spawn(async move { api::start(ui_enabled).await });
    }

    loop {
      if api_enabled {
        if let Ok(process) = Process::new(process::id()) {
          DAEMON_CPU_PERCENTAGE.observe(crate::service::get_process_cpu_usage_percentage(process.pid() as i64));
          DAEMON_MEM_USAGE.observe(process.memory_info().ok().unwrap().rss() as f64);
        }
      }

      then!(!Runner::new().is_empty(), restart_process());
      sleep(Duration::from_millis(config.interval));
    }
  }

  println!(
    "{} OMNITRON Successfully daemonized (type={})",
    *helpers::SUCCESS,
    global!("omnitron.daemon.kind")
  );
  if external {
    let callback = crate::Callback(init);
    crate::service::try_fork(false, verbose, callback);
  } else {
    match daemon(false, verbose) {
      Ok(Fork::Parent(_)) => {}
      Ok(Fork::Child) => init(),
      Err(err) => crashln!("{} Daemon creation failed with code {err}", *helpers::FAIL),
    }
  }
}

pub fn restart(api: &bool, webui: &bool, verbose: bool) {
  if pid::exists() {
    stop();
  }

  let config = config::read().daemon;

  if config.web.ui || *webui {
    ENABLE_API.store(true, Ordering::Release);
    ENABLE_WEBUI.store(true, Ordering::Release);
  } else if config.web.api {
    ENABLE_API.store(true, Ordering::Release);
  } else {
    ENABLE_API.store(*api, Ordering::Release);
  }

  start(verbose);
}

pub fn reset() {
  let mut runner = Runner::new();
  let largest = runner.size();

  match largest {
    Some(id) => runner.set_id(Id::from(str!(id.to_string()))),
    None => runner.set_id(Id::new(0)),
  }

  println!("{} Successfully reset (index={})", *helpers::SUCCESS, runner.id);
}

pub mod pid;
