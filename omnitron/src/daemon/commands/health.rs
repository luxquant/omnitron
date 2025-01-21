use chrono::{DateTime, Utc};
use colored::Colorize;
use global_placeholders::global;
use macros_rs::exp::ternary;
use macros_rs::fmt::string;
use omnitron_pm::helpers::ColoredString;
use omnitron_pm::process::Runner;
use omnitron_rpc::context;
use psutil::process::{MemoryInfo, Process};
use serde::Serialize;
use serde_json::json;
use tabled::settings::object::Columns;
use tabled::settings::style::{BorderColor, Style};
use tabled::settings::themes::Colorization;
use tabled::settings::{Color, Rotate};
use tabled::{Table, Tabled};

use crate::helpers::get_version;

pub(crate) async fn health(format: &String) {
  let mut pid: Option<i32> = None;
  let mut cpu_percent: Option<f64> = None;
  let mut uptime: Option<DateTime<Utc>> = None;
  let mut memory_usage: Option<MemoryInfo> = None;
  let mut runner: Runner = omnitron_pm::file::read_object(global!("omnitron.dump"));

  #[derive(Clone, Debug, Tabled)]
  struct Info {
    #[tabled(rename = "pid file")]
    pid_file: String,
    #[tabled(rename = "base path")]
    path: String,
    #[tabled(rename = "cpu percent")]
    cpu_percent: String,
    #[tabled(rename = "memory usage")]
    memory_usage: String,
    // #[tabled(rename = "daemon type")]
    // external: String,
    #[tabled(rename = "process count")]
    process_count: usize,
    uptime: String,
    pid: String,
    version: String,
    status: ColoredString,
  }

  impl Serialize for Info {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
      let trimmed_json = json!({
       "pid_file": &self.pid_file.trim(),
       "path": &self.path.trim(),
       "cpu": &self.cpu_percent.trim(),
       "mem": &self.memory_usage.trim(),
       "process_count": &self.process_count.to_string(),
       "uptime": &self.uptime.trim(),
       "pid": &self.pid.trim(),
       "version": &self.version.trim(),
       "status": &self.status.0.trim(),
      });

      trimmed_json.serialize(serializer)
    }
  }

  if omnitron_pm::daemon::pid::exists() {
    if let Ok(process_id) = omnitron_pm::daemon::pid::read() {
      if let Ok(process) = Process::new(process_id.get::<u32>()) {
        pid = Some(process.pid() as i32);
        uptime = Some(omnitron_pm::daemon::pid::uptime().unwrap());
        memory_usage = process.memory_info().ok();
        cpu_percent = Some(omnitron_pm::service::get_process_cpu_usage_percentage(
          process_id.get::<i64>(),
        ));
      }
    }
  }

  let cpu_percent = match cpu_percent {
    Some(percent) => format!("{:.2}%", percent),
    None => string!("0.00%"),
  };

  let memory_usage = match memory_usage {
    Some(usage) => omnitron_pm::helpers::format_memory(usage.rss()),
    None => string!("0b"),
  };

  let uptime = match uptime {
    Some(uptime) => omnitron_pm::helpers::format_duration(uptime),
    None => string!("none"),
  };

  let pid = match pid {
    Some(pid) => string!(pid),
    None => string!("n/a"),
  };

  let pid_exists = omnitron_pm::daemon::pid::exists();

  let version = if pid_exists {
    if let Ok(client) = crate::daemon::rpc::create_client().await {
      client.version(context::current(), false).await.unwrap()
    } else {
      get_version(false)
    }
  } else {
    get_version(false)
  };

  let data = vec![Info {
    pid: pid,
    cpu_percent,
    memory_usage,
    uptime: uptime,
    path: global!("omnitron.base"),
    // external: global!("omnitron.daemon.kind"),
    process_count: runner.count(),
    pid_file: format!("{}  ", global!("omnitron.pid")),
    version: version,
    status: ColoredString(ternary!(pid_exists, "online".green().bold(), "stopped".red().bold())),
  }];

  let table = Table::new(data.clone())
    .with(Rotate::Left)
    .with(Style::rounded().remove_horizontals())
    .with(Colorization::exact([Color::FG_CYAN], Columns::first()))
    .with(BorderColor::filled(Color::FG_BRIGHT_BLACK))
    .to_string();

  if let Ok(json) = serde_json::to_string(&data[0]) {
    match format.as_str() {
      "raw" => println!("{:?}", data[0]),
      "json" => println!("{json}"),
      "default" => {
        println!(
          "{}\n{table}\n",
          format!("OMNITRON daemon information").on_bright_white().black()
        );
        println!(" {}", format!("Use `omnitron daemon restart` to restart the daemon").white());
        println!(
          " {}",
          format!("Use `omnitron daemon reset` to clean process id values").white()
        );
      }
      _ => {}
    };
  };
}
