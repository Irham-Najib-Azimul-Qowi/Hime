use crate::routines::models::{RoutineSchedule, Weekday};

/// Parses a "HH:MM" 24h string into (hour, minute).
pub fn parse_time_hh_mm(t: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let h: u32 = parts[0].trim().parse().ok()?;
    let m: u32 = parts[1].trim().parse().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

/// Computes the day of week (0=Sunday, 1=Monday, ..., 6=Saturday) from epoch seconds.
pub fn weekday_from_epoch(now_secs: i64) -> Weekday {
    let days = now_secs / 86400;
    // 1970-01-01 was Thursday (index 4)
    let w_idx = ((days + 4) % 7 + 7) % 7;
    Weekday::from_index(w_idx as u32).unwrap_or(Weekday::Thursday)
}

/// Finds the next scheduled occurrence (epoch seconds) for a `RoutineSchedule` starting at or after `now_secs`.
/// If `allow_within_grace_secs` > 0, an occurrence today that started within the grace period is still eligible.
pub fn calculate_next_routine_start(
    schedule: &RoutineSchedule,
    now_secs: i64,
    allow_within_grace_secs: i64,
) -> Option<i64> {
    let (target_h, target_m) = parse_time_hh_mm(&schedule.time)?;
    let target_secs_into_day = (target_h as i64) * 3600 + (target_m as i64) * 60;

    let current_day_idx = now_secs / 86400;

    for day_offset in 0..=14 {
        let candidate_day = current_day_idx + day_offset;
        let w_idx = ((candidate_day + 4) % 7 + 7) % 7;
        let candidate_weekday = Weekday::from_index(w_idx as u32)?;

        if schedule.days.is_empty() || schedule.days.contains(&candidate_weekday) {
            let candidate_start = candidate_day * 86400 + target_secs_into_day;
            if candidate_start >= now_secs || (candidate_start + allow_within_grace_secs >= now_secs) {
                return Some(candidate_start);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time_hh_mm("19:30"), Some((19, 30)));
        assert_eq!(parse_time_hh_mm("00:00"), Some((0, 0)));
        assert_eq!(parse_time_hh_mm("25:00"), None);
        assert_eq!(parse_time_hh_mm("invalid"), None);
    }

    #[test]
    fn test_next_routine_start() {
        let schedule = RoutineSchedule {
            days: vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
            time: "10:00".to_string(),
            duration_min: 30,
            window_min: None,
        };

        // 1970-01-01 was Thursday, 00:00
        // Next match should be Friday 1970-01-02 10:00:00
        let next = calculate_next_routine_start(&schedule, 0, 0).unwrap();
        assert_eq!(next, 86400 + 10 * 3600);
    }
}
