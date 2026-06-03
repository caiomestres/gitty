use anyhow::{bail, Context, Result};
use gitty_core::config::Config;

use crate::NotificationAction;

pub fn cmd_notification(action: NotificationAction) -> Result<()> {
    match action {
        NotificationAction::Show => {
            let config = Config::load().context("loading config")?;
            let trigger = &config.notifications.trigger;
            let mode = match trigger {
                gitty_core::notification::NotificationTrigger::OnCritical => "on-critical",
                gitty_core::notification::NotificationTrigger::OnAnyChange => "on-any-change",
                gitty_core::notification::NotificationTrigger::OnSchedulerComplete => {
                    "on-scheduler-complete"
                }
                gitty_core::notification::NotificationTrigger::Disabled => "disabled",
            };
            println!("Notification trigger: {mode}");
            if let Some(interval) = config.notifications.polling_interval_minutes {
                println!("Polling interval:     {interval} minutes");
            }
            println!(
                "History:              {} notifications",
                config.notification_history.len()
            );
        }
        NotificationAction::Set { mode } => {
            let trigger = match mode.as_str() {
                "on-critical" => gitty_core::notification::NotificationTrigger::OnCritical,
                "on-any-change" => gitty_core::notification::NotificationTrigger::OnAnyChange,
                "on-scheduler-complete" => {
                    gitty_core::notification::NotificationTrigger::OnSchedulerComplete
                }
                "disabled" => gitty_core::notification::NotificationTrigger::Disabled,
                other => bail!(
                    "unknown trigger mode '{other}'. Use: on-critical, on-any-change, on-scheduler-complete, disabled"
                ),
            };
            let mut config = Config::load().context("loading config")?;
            config.notifications.trigger = trigger;
            config.save().context("saving config")?;
            println!("Notification trigger set to '{mode}'");
        }
    }
    Ok(())
}
