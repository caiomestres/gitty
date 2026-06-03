use anyhow::{Context, Result};
use gitty_core::config::Config;

use crate::NotificationAction;

fn parse_trigger(s: &str) -> Option<gitty_core::notification::NotificationTrigger> {
    use gitty_core::notification::NotificationTrigger;
    match s {
        "on_critical" | "on-critical" => Some(NotificationTrigger::OnCritical),
        "on_any_change" | "on-any-change" => Some(NotificationTrigger::OnAnyChange),
        "on_scheduler_complete" | "on-scheduler-complete" => {
            Some(NotificationTrigger::OnSchedulerComplete)
        }
        "disabled" => Some(NotificationTrigger::Disabled),
        _ => None,
    }
}

fn trigger_display(trigger: &gitty_core::notification::NotificationTrigger) -> &'static str {
    use gitty_core::notification::NotificationTrigger;
    match trigger {
        NotificationTrigger::OnCritical => "on_critical",
        NotificationTrigger::OnAnyChange => "on_any_change",
        NotificationTrigger::OnSchedulerComplete => "on_scheduler_complete",
        NotificationTrigger::Disabled => "disabled",
    }
}

pub fn cmd_notification(action: NotificationAction) -> Result<()> {
    match action {
        NotificationAction::Show => {
            let config = Config::load().context("loading config")?;
            let config_dir =
                gitty_core::config::paths::config_dir().context("resolving config dir")?;
            let mode = trigger_display(&config.notifications.trigger);
            println!("Notification trigger: {mode}");
            if let Some(interval) = config.notifications.polling_interval_minutes {
                println!("Polling interval:     {interval} minutes");
            }
            let history = gitty_core::notification::load_history(&config_dir);
            println!(
                "History:              {} notifications",
                history.len()
            );
        }
        NotificationAction::Set { mode } => {
            let trigger = parse_trigger(&mode).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown trigger mode '{mode}'. Use: on_critical, on_any_change, on_scheduler_complete, disabled"
                )
            })?;
            let mut config = Config::load().context("loading config")?;
            config.notifications.trigger = trigger;
            config.save().context("saving config")?;
            println!("Notification trigger set to '{mode}'");
        }
    }
    Ok(())
}
