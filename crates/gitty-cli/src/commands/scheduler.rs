use anyhow::{Context, Result};
use gitty_core::config::Config;
use time::format_description::well_known::Rfc3339;

use crate::SchedulerAction;

pub fn cmd_scheduler(action: SchedulerAction) -> Result<()> {
    let config_dir = gitty_core::config::paths::config_dir().context("resolving config dir")?;

    match action {
        SchedulerAction::Start => match gitty_core::scheduler::daemon::start_daemon(&config_dir) {
            Ok(()) => {
                println!("Scheduler started.");
            }
            Err(e) => {
                eprintln!("Failed to start scheduler: {e}");
            }
        },
        SchedulerAction::RunDaemon => {
            gitty_core::scheduler::daemon::run_foreground(&config_dir)
                .context("running scheduler daemon")?;
        }
        SchedulerAction::Stop => {
            let stopped =
                gitty_core::scheduler::daemon::stop(&config_dir).context("stopping scheduler")?;
            if stopped {
                println!("Scheduler stopped.");
            } else {
                println!("Scheduler is not running.");
            }
        }
        SchedulerAction::Status => {
            let status = gitty_core::scheduler::daemon::status(&config_dir);
            if status.running {
                println!("Scheduler: running (PID {})", status.pid.unwrap_or(0));
            } else {
                println!("Scheduler: stopped");
            }
            let config = Config::load().context("loading config")?;
            if let Some(lr) = &config.scheduler.last_run {
                println!("  Last run: {}", lr.format(&Rfc3339).unwrap_or_default());
            }
            if let Some(nr) = &config.scheduler.next_run {
                println!("  Next run: {}", nr.format(&Rfc3339).unwrap_or_default());
            }
        }
    }
    Ok(())
}
