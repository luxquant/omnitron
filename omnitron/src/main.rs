#![feature(type_alias_impl_trait)]

mod daemon;
mod gate;
mod globals;
mod helpers;
mod logging;

use anyhow::{Ok, Result};
use clap::{ArgAction, Parser};
use gate::config::load_config;
use logging::init_logging;
use macros_rs::fmt::{str, string};
use tracing::*;

#[derive(clap::Parser)]
#[clap(author, version =str!(helpers::get_version(false)), about, long_about = None, arg_required_else_help = true, propagate_version = true)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,

  #[clap(long, short, action=ArgAction::Count)]
  debug: u8,
}

#[derive(clap::Subcommand)]
enum Commands {
  /// SSH/HTTP/MySQL/PostgreSQL/etc gate.
  #[command(visible_alias = "gt")]
  Gate {
    #[command(subcommand)]
    command: GateCommands,
  },
  /// Process manager
  Pm {
    #[command(subcommand)]
    command: PmCommands,
  },
  /// Get daemon info
  #[command(visible_alias = "health", visible_alias = "status")]
  Info {
    /// Format output
    #[arg(long, default_value_t = string!("default"))]
    format: String,
  },
  /// Start daemon
  #[command(visible_alias = "start")]
  Up {
    /// Enable an API token (passed via the `OMNITRON_ADMIN_TOKEN` env var) that automatically maps to the first admin user
    #[clap(long, action=ArgAction::SetTrue)]
    enable_admin_token: bool,
  },
  // /// Reset process index
  // #[command(visible_alias = "reset_position")]
  // Reset,
  /// Stop daemon
  #[command(visible_alias = "kill", visible_alias = "stop")]
  Down,
  // /// Restart daemon
  // #[command(visible_alias = "restart", visible_alias = "start")]
  // Restore {
  //   /// Daemon api
  //   #[arg(long)]
  //   api: bool,
  //   /// WebUI using api
  //   #[arg(long)]
  //   webui: bool,
  // },
  // /// Check daemon health
  // #[command(visible_alias = "info", visible_alias = "status")]
  // Health {
  //   /// Format output
  //   #[arg(long, default_value_t = string!("default"))]
  //   format: String,
  // },
}

#[derive(clap::Subcommand)]
pub(crate) enum GateCommands {
  /// Run Warpgate
  Run {
    /// Enable an API token (passed via the `WARPGATE_ADMIN_TOKEN` env var) that automatically maps to the first admin user
    #[clap(long, action=ArgAction::SetTrue)]
    enable_admin_token: bool,
  },
  /// Show Omnitron's SSH client keys
  ClientKeys,
  /// Perform basic config checks
  Check,
  /// Test the connection to a target host
  TestTarget {
    #[clap(action=ArgAction::Set)]
    target_name: String,
  },
  /// Reset password and auth policy for a user
  RecoverAccess {
    #[clap(action=ArgAction::Set)]
    username: Option<String>,
  },
}

#[derive(clap::Subcommand)]
enum PmDaemon {
  /// Reset process index
  #[command(visible_alias = "reset_position")]
  Reset,
  /// Restart daemon
  #[command(visible_alias = "restart", visible_alias = "start")]
  Restore {
    /// Daemon api
    #[arg(long)]
    api: bool,
    /// WebUI using api
    #[arg(long)]
    webui: bool,
  },
}

#[derive(clap::Subcommand)]
enum Server {
  /// Add new server
  #[command(visible_alias = "add")]
  New,
  /// List servers
  #[command(visible_alias = "ls")]
  List {
    /// Format output
    #[arg(long, default_value_t = string!("default"))]
    format: String,
  },
  /// Remove server
  #[command(visible_alias = "rm", visible_alias = "delete")]
  Remove {
    /// Server name
    name: String,
  },
  /// Set default server
  #[command(visible_alias = "set")]
  Default {
    /// Server name
    name: Option<String>,
  },
}

// add omnitron restore command
#[derive(clap::Subcommand)]
enum PmCommands {
  /// Import process from environment file
  #[command(visible_alias = "add")]
  Import {
    /// Path of file to import
    path: String,
  },
  /// Export environment file from process
  #[command(visible_alias = "get")]
  Export {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    /// Path to export file
    path: Option<String>,
  },
  /// Start/Restart a process
  #[command(visible_alias = "restart")]
  Start {
    /// Process name
    #[arg(long)]
    name: Option<String>,
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Args>)]
    args: omnitron_pm::cli::Args,
    /// Watch to reload path
    #[arg(long)]
    watch: Option<String>,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
    /// Reset environment values
    #[arg(short, long)]
    reset_env: bool,
  },
  /// Stop/Kill a process
  #[command(visible_alias = "kill")]
  Stop {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Stop then remove a process
  #[command(visible_alias = "rm", visible_alias = "delete")]
  Remove {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Get env of a process
  #[command(visible_alias = "cmdline")]
  Env {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Get information of a process
  #[command(visible_alias = "info")]
  Details {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    /// Format output
    #[arg(long, default_value_t = string!("default"))]
    format: String,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// List all processes
  #[command(visible_alias = "ls")]
  List {
    /// Format output
    #[arg(long, default_value_t = string!("default"))]
    format: String,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Restore all processes
  #[command(visible_alias = "resurrect")]
  Restore {
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Save all processes to dumpfile
  #[command(visible_alias = "store")]
  Save {
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Get logs from a process
  Logs {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    #[arg(long, default_value_t = 15, help = "")]
    lines: usize,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Flush a process log
  #[command(visible_alias = "clean", visible_alias = "log_rotate")]
  Flush {
    #[clap(value_parser = omnitron_pm::cli::validate::<omnitron_pm::cli::Item>)]
    item: omnitron_pm::cli::Item,
    /// Server
    #[arg(short, long)]
    server: Option<String>,
  },
  /// Daemon management
  #[command(visible_alias = "agent", visible_alias = "bgd")]
  Daemon {
    #[command(subcommand)]
    command: PmDaemon,
  },

  /// Server management
  #[command(visible_alias = "remote", visible_alias = "srv")]
  Server {
    #[command(subcommand)]
    command: Server,
  },
}

async fn run(cli: &Cli) -> Result<()> {
  init_logging(load_config(false).ok().as_ref(), cli).await;

  match &cli.command {
    Commands::Down => {
      daemon::commands::stop();
      Ok(())
    }
    Commands::Info { format } => {
      daemon::commands::info(format).await;
      Ok(())
    }
    Commands::Gate { command } => match command {
      GateCommands::Run { enable_admin_token } => gate::commands::run::command(*enable_admin_token).await,
      GateCommands::Check => gate::commands::check::command(cli).await,
      GateCommands::TestTarget { target_name } => gate::commands::test_target::command(cli, target_name).await,
      GateCommands::ClientKeys => gate::commands::client_keys::command(cli).await,
      GateCommands::RecoverAccess { username } => gate::commands::recover_access::command(cli, username).await,
    },
    Commands::Pm { command } => match command {
      PmCommands::Import { path } => {
        omnitron_pm::cli::import::read_hcl(path);
        Ok(())
      }
      PmCommands::Export { item, path } => {
        omnitron_pm::cli::import::export_hcl(item, path);
        Ok(())
      }
      PmCommands::Start {
        name,
        args,
        watch,
        server,
        reset_env,
      } => {
        omnitron_pm::cli::start(name, args, watch, reset_env, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Stop { item, server } => {
        omnitron_pm::cli::stop(item, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Remove { item, server } => {
        omnitron_pm::cli::remove(item, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Restore { server } => {
        omnitron_pm::cli::internal::Internal::restore(&omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Save { server } => {
        omnitron_pm::cli::internal::Internal::save(&omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Env { item, server } => {
        omnitron_pm::cli::env(item, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Details { item, format, server } => {
        omnitron_pm::cli::info(item, format, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::List { format, server } => {
        omnitron_pm::cli::internal::Internal::list(format, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Logs { item, lines, server } => {
        omnitron_pm::cli::logs(item, lines, &omnitron_pm::globals::defaults(server));

        Ok(())
      }
      PmCommands::Flush { item, server } => {
        omnitron_pm::cli::flush(item, &omnitron_pm::globals::defaults(server));

        Ok(())
      }

      PmCommands::Daemon { command } => match command {
        PmDaemon::Reset => {
          omnitron_pm::daemon::reset();

          Ok(())
        }
        PmDaemon::Restore { api, webui } => {
          omnitron_pm::daemon::restart(api, webui, false /*level.as_str() != "OFF"*/);

          Ok(())
        }
      },

      PmCommands::Server { command } => match command {
        Server::New => {
          omnitron_pm::cli::server::new();

          Ok(())
        }
        Server::Remove { name } => {
          omnitron_pm::cli::server::remove(name);

          Ok(())
        }
        Server::Default { name } => {
          omnitron_pm::cli::server::default(name);

          Ok(())
        }
        Server::List { format } => {
          omnitron_pm::cli::server::list(format);

          Ok(())
        }
      },
    },
    _ => Ok(()),
  }
}

fn main() {
  globals::init();

  let cli = Cli::parse();
  if let Commands::Up { enable_admin_token } = &cli.command {
    daemon::commands::start(&cli, *enable_admin_token).unwrap();
  } else {
    tokio_main(&cli).unwrap();
  }
}

#[tokio::main]
async fn tokio_main(cli: &Cli) -> anyhow::Result<()> {
  if let Err(error) = run(cli).await {
    error!(?error, "Fatal error");
    std::process::exit(1);
  }

  Ok(())
}
