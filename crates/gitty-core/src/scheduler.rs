pub mod daemon;
pub mod runner;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    #[serde(default)]
    pub macro_id: Option<Uuid>,
    #[serde(default)]
    pub trigger: SchedulerTrigger,
    #[serde(default)]
    pub power: PowerConfig,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_run: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub next_run: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DayOfWeek {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl DayOfWeek {
    pub fn from_weekday(wd: time::Weekday) -> Self {
        match wd {
            time::Weekday::Monday => Self::Mon,
            time::Weekday::Tuesday => Self::Tue,
            time::Weekday::Wednesday => Self::Wed,
            time::Weekday::Thursday => Self::Thu,
            time::Weekday::Friday => Self::Fri,
            time::Weekday::Saturday => Self::Sat,
            time::Weekday::Sunday => Self::Sun,
        }
    }

    pub fn to_weekday(self) -> time::Weekday {
        match self {
            Self::Mon => time::Weekday::Monday,
            Self::Tue => time::Weekday::Tuesday,
            Self::Wed => time::Weekday::Wednesday,
            Self::Thu => time::Weekday::Thursday,
            Self::Fri => time::Weekday::Friday,
            Self::Sat => time::Weekday::Saturday,
            Self::Sun => time::Weekday::Sunday,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

impl TimeOfDay {
    pub fn minutes_since_midnight(self) -> u16 {
        self.hour as u16 * 60 + self.minute as u16
    }

    pub fn from_hm(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum SchedulerTrigger {
    Simple {
        interval_minutes: u32,
    },
    Advanced {
        interval_minutes: u32,
        window_start: TimeOfDay,
        window_end: TimeOfDay,
        days: Vec<DayOfWeek>,
    },
}

impl Default for SchedulerTrigger {
    fn default() -> Self {
        Self::Simple {
            interval_minutes: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    pub pause_on_battery: bool,
    pub battery_threshold: u8,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            pause_on_battery: true,
            battery_threshold: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub running: bool,
    pub pid: Option<u32>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_run: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub next_run: Option<OffsetDateTime>,
}

// ---------------------------------------------------------------------------
// Trigger Logic
// ---------------------------------------------------------------------------

fn in_time_window(now_minutes: u16, start: TimeOfDay, end: TimeOfDay) -> bool {
    let s = start.minutes_since_midnight();
    let e = end.minutes_since_midnight();
    if s <= e {
        now_minutes >= s && now_minutes <= e
    } else {
        // Midnight-crossing window (e.g., 22:00-06:00)
        now_minutes >= s || now_minutes <= e
    }
}

/// Pure function: returns `true` when a scheduler run should execute now.
pub fn should_run(
    config: &SchedulerConfig,
    now: OffsetDateTime,
    on_battery: bool,
    battery_level: u8,
) -> bool {
    if !config.enabled {
        return false;
    }

    if config.power.pause_on_battery && on_battery && battery_level < config.power.battery_threshold
    {
        return false;
    }

    match &config.trigger {
        SchedulerTrigger::Simple { interval_minutes } => match config.last_run {
            None => true,
            Some(lr) => (now - lr).whole_minutes() >= *interval_minutes as i64,
        },
        SchedulerTrigger::Advanced {
            interval_minutes,
            window_start,
            window_end,
            days,
        } => {
            let today = DayOfWeek::from_weekday(now.weekday());
            if !days.contains(&today) {
                return false;
            }

            let now_minutes = now.hour() as u16 * 60 + now.minute() as u16;
            if !in_time_window(now_minutes, *window_start, *window_end) {
                return false;
            }

            match config.last_run {
                None => true,
                Some(lr) => (now - lr).whole_minutes() >= *interval_minutes as i64,
            }
        }
    }
}

/// Update `last_run` and compute `next_run`.
pub fn record_run(config: &mut SchedulerConfig, now: OffsetDateTime) {
    config.last_run = Some(now);
    config.next_run = compute_next_run(config, now);
}

/// Compute the next valid run time from `from`.
/// For Advanced mode, scans forward respecting window and day constraints (7-day cap).
pub fn compute_next_run(config: &SchedulerConfig, from: OffsetDateTime) -> Option<OffsetDateTime> {
    if !config.enabled {
        return None;
    }
    match &config.trigger {
        SchedulerTrigger::Simple { interval_minutes } => {
            Some(from + time::Duration::minutes(*interval_minutes as i64))
        }
        SchedulerTrigger::Advanced {
            interval_minutes,
            window_start,
            window_end,
            days,
        } => {
            let mut candidate = from + time::Duration::minutes(*interval_minutes as i64);
            let cap = from + time::Duration::days(7);

            while candidate <= cap {
                let day = DayOfWeek::from_weekday(candidate.weekday());
                let mins = candidate.hour() as u16 * 60 + candidate.minute() as u16;
                if days.contains(&day) && in_time_window(mins, *window_start, *window_end) {
                    return Some(candidate);
                }
                candidate += time::Duration::minutes(1);
            }
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    fn config_simple(interval: u32, last: Option<OffsetDateTime>) -> SchedulerConfig {
        SchedulerConfig {
            enabled: true,
            macro_id: None,
            trigger: SchedulerTrigger::Simple {
                interval_minutes: interval,
            },
            power: PowerConfig::default(),
            last_run: last,
            next_run: None,
        }
    }

    #[test]
    fn should_run_first_time() {
        let config = config_simple(30, None);
        assert!(should_run(&config, now(), false, 100));
    }

    #[test]
    fn should_run_after_interval() {
        let past = now() - time::Duration::minutes(31);
        let config = config_simple(30, Some(past));
        assert!(should_run(&config, now(), false, 100));
    }

    #[test]
    fn should_not_run_before_interval() {
        let recent = now() - time::Duration::minutes(10);
        let config = config_simple(30, Some(recent));
        assert!(!should_run(&config, now(), false, 100));
    }

    #[test]
    fn should_not_run_when_disabled() {
        let mut config = config_simple(30, None);
        config.enabled = false;
        assert!(!should_run(&config, now(), false, 100));
    }

    #[test]
    fn should_not_run_on_battery_below_threshold() {
        let config = config_simple(30, None);
        assert!(!should_run(&config, now(), true, 15));
    }

    #[test]
    fn should_run_on_battery_above_threshold() {
        let config = config_simple(30, None);
        assert!(should_run(&config, now(), true, 50));
    }

    #[test]
    fn should_run_ac_power_ignores_battery() {
        let config = config_simple(30, None);
        assert!(should_run(&config, now(), false, 5));
    }

    #[test]
    fn advanced_trigger_inside_window() {
        let n = now();
        let start = TimeOfDay::from_hm(n.hour(), 0);
        let end = TimeOfDay::from_hm(n.hour(), 59);
        let today = DayOfWeek::from_weekday(n.weekday());

        let config = SchedulerConfig {
            enabled: true,
            macro_id: None,
            trigger: SchedulerTrigger::Advanced {
                interval_minutes: 1,
                window_start: start,
                window_end: end,
                days: vec![today],
            },
            power: PowerConfig::default(),
            last_run: Some(n - time::Duration::minutes(5)),
            next_run: None,
        };

        assert!(should_run(&config, n, false, 100));
    }

    #[test]
    fn advanced_trigger_wrong_day() {
        let n = now();
        let config = SchedulerConfig {
            enabled: true,
            macro_id: None,
            trigger: SchedulerTrigger::Advanced {
                interval_minutes: 1,
                window_start: TimeOfDay::from_hm(0, 0),
                window_end: TimeOfDay::from_hm(23, 59),
                days: vec![], // no valid days
            },
            power: PowerConfig::default(),
            last_run: None,
            next_run: None,
        };
        assert!(!should_run(&config, n, false, 100));
    }

    #[test]
    fn record_run_updates_fields() {
        let mut config = config_simple(30, None);
        let n = now();
        record_run(&mut config, n);
        assert!(config.last_run.is_some());
        assert!(config.next_run.is_some());
    }

    #[test]
    fn compute_next_run_disabled_returns_none() {
        let mut config = config_simple(30, None);
        config.enabled = false;
        assert!(compute_next_run(&config, now()).is_none());
    }

    #[test]
    fn midnight_crossing_window() {
        let start = TimeOfDay::from_hm(22, 0);
        let end = TimeOfDay::from_hm(6, 0);
        assert!(in_time_window(23 * 60, start, end)); // 23:00
        assert!(in_time_window(2 * 60, start, end)); // 02:00
        assert!(!in_time_window(12 * 60, start, end)); // 12:00 (outside)
    }

    #[test]
    fn normal_window() {
        let start = TimeOfDay::from_hm(9, 0);
        let end = TimeOfDay::from_hm(17, 0);
        assert!(in_time_window(12 * 60, start, end)); // 12:00
        assert!(!in_time_window(20 * 60, start, end)); // 20:00
    }

    #[test]
    fn compute_next_run_advanced_respects_window() {
        let n = now();
        let config = SchedulerConfig {
            enabled: true,
            macro_id: None,
            trigger: SchedulerTrigger::Advanced {
                interval_minutes: 30,
                window_start: TimeOfDay::from_hm(9, 0),
                window_end: TimeOfDay::from_hm(17, 0),
                days: vec![
                    DayOfWeek::Mon,
                    DayOfWeek::Tue,
                    DayOfWeek::Wed,
                    DayOfWeek::Thu,
                    DayOfWeek::Fri,
                    DayOfWeek::Sat,
                    DayOfWeek::Sun,
                ],
            },
            power: PowerConfig::default(),
            last_run: None,
            next_run: None,
        };

        let next = compute_next_run(&config, n);
        if let Some(next_time) = next {
            let mins = next_time.hour() as u16 * 60 + next_time.minute() as u16;
            assert!(in_time_window(
                mins,
                TimeOfDay::from_hm(9, 0),
                TimeOfDay::from_hm(17, 0)
            ));
        }
    }

    #[test]
    fn scheduler_config_serde_round_trip() {
        let config = config_simple(30, Some(now()));
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SchedulerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enabled, config.enabled);
        assert!(parsed.last_run.is_some());
    }
}
