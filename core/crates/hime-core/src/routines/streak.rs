use crate::routines::models::{RoutineSchedule, RoutineStreak, Weekday};

/// Converts a "YYYY-MM-DD" string to civil days since Unix epoch.
pub fn date_str_to_days(d: &str) -> Option<i64> {
    let parts: Vec<&str> = d.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;

    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + (doy as u32);
    Some(era * 146097 + (doe as i64) - 719468)
}

/// Converts civil days since Unix epoch to "YYYY-MM-DD".
/// Uses canonical Howard Hinnant algorithm.
pub fn days_to_date_str(days: i64) -> String {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Returns whether the given civil day (epoch days) is a scheduled day for the routine.
/// If `schedule` is None or has empty days, every day is considered scheduled (daily routine).
pub fn is_day_scheduled(day: i64, schedule: Option<&RoutineSchedule>) -> bool {
    match schedule {
        Some(s) if !s.days.is_empty() => {
            let w_idx = ((day + 4).rem_euclid(7)) as u32;
            if let Some(weekday) = Weekday::from_index(w_idx) {
                s.days.contains(&weekday)
            } else {
                true
            }
        }
        _ => true,
    }
}

/// Streak engine logic with schedule awareness:
/// - Increments streak on routine completion.
/// - Prevents double-increment if completed multiple times on the same date.
/// - Accounts for consecutive scheduled occurrences or resuming after a frozen (excused) date.
/// - For non-daily routines (e.g. Mon/Wed/Fri), intermediate unscheduled days do NOT break streak.
/// - If any scheduled day between last completion/freeze and today was missed without excuse, resets streak count to 1.
pub fn apply_streak_completion_with_schedule(
    streak: &mut RoutineStreak,
    today_str: &str,
    schedule: Option<&RoutineSchedule>,
) {
    let today_days = match date_str_to_days(today_str) {
        Some(d) => d,
        None => return,
    };

    // If already completed today, streak remains unchanged (idempotent)
    if streak.last_completed_date.as_deref() == Some(today_str) {
        return;
    }

    let frozen_days = streak.frozen_on_date.as_deref().and_then(date_str_to_days);

    let is_consecutive = if let Some(ref last_date) = streak.last_completed_date {
        let last_days = date_str_to_days(last_date).unwrap_or(0);
        if today_days > last_days {
            // Determine reference start day: use frozen_days if after last completion, else last_days
            let start_days = match frozen_days {
                Some(fd) if fd > last_days && fd < today_days => fd,
                _ => last_days,
            };

            let mut missed_scheduled = false;
            for d in (start_days + 1)..today_days {
                if is_day_scheduled(d, schedule) && frozen_days != Some(d) {
                    missed_scheduled = true;
                    break;
                }
            }
            !missed_scheduled
        } else {
            false
        }
    } else {
        false
    };

    if is_consecutive {
        streak.current_count += 1;
    } else {
        // Gap, missed scheduled occurrence, or starting fresh
        streak.current_count = 1;
    }

    if streak.current_count > streak.best_count {
        streak.best_count = streak.current_count;
    }

    streak.last_completed_date = Some(today_str.to_string());
    streak.frozen_on_date = None; // Unfreeze once completed
}

/// Convenience wrapper for daily routines or callers without schedule object.
pub fn apply_streak_completion(streak: &mut RoutineStreak, today_str: &str) {
    apply_streak_completion_with_schedule(streak, today_str, None);
}

/// Excused skip freezes the streak (e.g. Sakit, Libur, Sibuk).
/// Current streak is preserved without penalty.
pub fn apply_streak_excused_skip(streak: &mut RoutineStreak, today_str: &str) {
    // If not already frozen or completed today, record freeze date
    streak.frozen_on_date = Some(today_str.to_string());
}

/// Unexcused skip or missed routine resets current streak to 0.
pub fn apply_streak_miss(streak: &mut RoutineStreak) {
    streak.current_count = 0;
    streak.frozen_on_date = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_conversion_roundtrip() {
        assert_eq!(days_to_date_str(0), "1970-01-01");
        assert_eq!(date_str_to_days("1970-01-01"), Some(0));

        let date = "2026-11-21";
        let days = date_str_to_days(date).unwrap();
        assert_eq!(days_to_date_str(days), date);
    }

    #[test]
    fn test_civil_date_roundtrip_30000_days() {
        for day in 0..30000 {
            let s = days_to_date_str(day);
            let roundtrip = date_str_to_days(&s).expect("valid date string");
            assert_eq!(roundtrip, day, "Failed roundtrip for day {}", day);
        }
    }

    #[test]
    fn test_civil_date_critical_boundaries() {
        // Explicitly asserting required boundary and leap year dates
        assert_eq!(days_to_date_str(date_str_to_days("2026-02-28").unwrap()), "2026-02-28");
        assert_eq!(days_to_date_str(date_str_to_days("2026-03-01").unwrap()), "2026-03-01");
        assert_eq!(days_to_date_str(date_str_to_days("2024-02-29").unwrap()), "2024-02-29");
        assert_eq!(days_to_date_str(date_str_to_days("2000-02-29").unwrap()), "2000-02-29");
    }

    #[test]
    fn test_streak_consecutive_increments() {
        let mut streak = RoutineStreak::default();
        apply_streak_completion(&mut streak, "2026-11-20");
        assert_eq!(streak.current_count, 1);
        assert_eq!(streak.best_count, 1);

        // Same day completion does not double-increment
        apply_streak_completion(&mut streak, "2026-11-20");
        assert_eq!(streak.current_count, 1);

        // Next day consecutive
        apply_streak_completion(&mut streak, "2026-11-21");
        assert_eq!(streak.current_count, 2);
        assert_eq!(streak.best_count, 2);
    }

    #[test]
    fn test_streak_non_daily_schedule() {
        let schedule = RoutineSchedule {
            days: vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
            time: "09:00".to_string(),
            duration_min: 30,
            window_min: None,
        };

        let mut streak = RoutineStreak::default();

        // Monday 2026-11-02 completion -> streak 1
        apply_streak_completion_with_schedule(&mut streak, "2026-11-02", Some(&schedule));
        assert_eq!(streak.current_count, 1);
        assert_eq!(streak.best_count, 1);

        // Wednesday 2026-11-04 completion -> streak 2 (Tuesday was not scheduled)
        apply_streak_completion_with_schedule(&mut streak, "2026-11-04", Some(&schedule));
        assert_eq!(streak.current_count, 2);
        assert_eq!(streak.best_count, 2);

        // Friday 2026-11-06 completion -> streak 3 (Thursday was not scheduled)
        apply_streak_completion_with_schedule(&mut streak, "2026-11-06", Some(&schedule));
        assert_eq!(streak.current_count, 3);
        assert_eq!(streak.best_count, 3);

        // Monday 2026-11-09 completion -> streak 4 (Saturday and Sunday not scheduled)
        apply_streak_completion_with_schedule(&mut streak, "2026-11-09", Some(&schedule));
        assert_eq!(streak.current_count, 4);
        assert_eq!(streak.best_count, 4);
    }

    #[test]
    fn test_streak_non_daily_schedule_miss_resets() {
        let schedule = RoutineSchedule {
            days: vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
            time: "09:00".to_string(),
            duration_min: 30,
            window_min: None,
        };

        let mut streak = RoutineStreak::default();

        // Monday 2026-11-02 completion -> streak 1
        apply_streak_completion_with_schedule(&mut streak, "2026-11-02", Some(&schedule));
        assert_eq!(streak.current_count, 1);

        // User misses Wednesday 2026-11-04 and completes Friday 2026-11-06 without excuse -> reset to 1
        apply_streak_completion_with_schedule(&mut streak, "2026-11-06", Some(&schedule));
        assert_eq!(streak.current_count, 1);
        assert_eq!(streak.best_count, 1);
    }

    #[test]
    fn test_streak_non_daily_schedule_with_excused_skip() {
        let schedule = RoutineSchedule {
            days: vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
            time: "09:00".to_string(),
            duration_min: 30,
            window_min: None,
        };

        let mut streak = RoutineStreak::default();

        // Monday 2026-11-02 completion -> streak 1
        apply_streak_completion_with_schedule(&mut streak, "2026-11-02", Some(&schedule));
        assert_eq!(streak.current_count, 1);

        // Wednesday 2026-11-04 excused skip
        apply_streak_excused_skip(&mut streak, "2026-11-04");
        assert_eq!(streak.current_count, 1);

        // Friday 2026-11-06 completion -> streak increments to 2
        apply_streak_completion_with_schedule(&mut streak, "2026-11-06", Some(&schedule));
        assert_eq!(streak.current_count, 2);
    }

    #[test]
    fn test_streak_frozen_excused_skip() {
        let mut streak = RoutineStreak {
            current_count: 5,
            best_count: 10,
            last_completed_date: Some("2026-11-20".to_string()),
            frozen_on_date: None,
        };

        // Excused skip on day 21 (Sakit/Libur) freezes streak
        apply_streak_excused_skip(&mut streak, "2026-11-21");
        assert_eq!(streak.current_count, 5);
        assert_eq!(streak.frozen_on_date.as_deref(), Some("2026-11-21"));

        // Complete on day 22 resumes consecutive streak: 5 -> 6
        apply_streak_completion(&mut streak, "2026-11-22");
        assert_eq!(streak.current_count, 6);
        assert_eq!(streak.frozen_on_date, None);
    }

    #[test]
    fn test_streak_unexcused_miss_resets() {
        let mut streak = RoutineStreak {
            current_count: 5,
            best_count: 10,
            last_completed_date: Some("2026-11-20".to_string()),
            frozen_on_date: None,
        };

        apply_streak_miss(&mut streak);
        assert_eq!(streak.current_count, 0);
        assert_eq!(streak.best_count, 10); // Best count preserved
    }
}
