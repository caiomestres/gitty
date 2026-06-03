/// Query the system battery state.
/// Returns (on_battery, battery_level_percent).
/// If no battery is detected or the crate fails, returns (false, 100) — "always on AC".
pub fn battery_state() -> (bool, u8) {
    match battery::Manager::new() {
        Ok(manager) => {
            let mut batteries = match manager.batteries() {
                Ok(b) => b,
                Err(_) => return (false, 100),
            };
            match batteries.next() {
                Some(Ok(bat)) => {
                    let level =
                        (bat.state_of_charge()
                            .get::<battery::units::ratio::percent>()) as u8;
                    let on_battery = bat.state() == battery::State::Discharging;
                    (on_battery, level)
                }
                _ => (false, 100),
            }
        }
        Err(_) => (false, 100),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_state_returns_valid_values() {
        let (on_battery, level) = battery_state();
        // On desktops without batteries, should return (false, 100)
        assert!(level <= 100);
        let _ = on_battery; // just ensure no panic
    }
}
