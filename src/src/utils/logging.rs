// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

fn format_event_message(event: &str, phase: &str, state: &str, details: Option<&str>) -> String {
    match details {
        Some(d) if !d.is_empty() => format!("event={event} phase={phase} state={state} {d}"),
        _ => format!("event={event} phase={phase} state={state}"),
    }
}

fn format_timing_message(
    event: &str,
    phase: &str,
    state: &str,
    duration: std::time::Duration,
    details: Option<&str>,
) -> String {
    let elapsed_ms = duration.as_secs_f64() * 1000.0;
    match details {
        Some(d) if !d.is_empty() => {
            format!("event={event} phase={phase} state={state} elapsed_ms={elapsed_ms:.1} {d}")
        }
        _ => format!("event={event} phase={phase} state={state} elapsed_ms={elapsed_ms:.1}"),
    }
}

pub fn log_info_evt(event: &str, phase: &str, state: &str, details: Option<&str>) {
    log::info!("{}", format_event_message(event, phase, state, details));
}

pub fn log_debug_evt(event: &str, phase: &str, state: &str, details: Option<&str>) {
    log::debug!("{}", format_event_message(event, phase, state, details));
}

pub fn log_warn_evt(event: &str, phase: &str, state: &str, details: Option<&str>) {
    log::warn!("{}", format_event_message(event, phase, state, details));
}

pub fn log_timing_evt(
    event: &str,
    phase: &str,
    state: &str,
    duration: std::time::Duration,
    details: Option<&str>,
) {
    log::info!(
        "{}",
        format_timing_message(event, phase, state, duration, details)
    );
}

#[cfg(test)]
mod tests {
    use super::{format_event_message, format_timing_message};
    use std::time::Duration;

    #[test]
    fn format_event_message_without_details() {
        let msg = format_event_message("search", "start", "begin", None);
        assert_eq!(msg, "event=search phase=start state=begin");
    }

    #[test]
    fn format_event_message_ignores_empty_details() {
        let msg = format_event_message("search", "start", "begin", Some(""));
        assert_eq!(msg, "event=search phase=start state=begin");
    }

    #[test]
    fn format_event_message_appends_details() {
        let msg = format_event_message("download", "dispatch", "started", Some("format=csv"));
        assert_eq!(
            msg,
            "event=download phase=dispatch state=started format=csv"
        );
    }

    #[test]
    fn format_timing_message_without_details() {
        let msg = format_timing_message(
            "search",
            "complete",
            "done",
            Duration::from_secs_f64(1.234),
            None,
        );
        assert_eq!(
            msg,
            "event=search phase=complete state=done elapsed_ms=1234.0"
        );
    }

    #[test]
    fn format_timing_message_with_details() {
        let msg = format_timing_message(
            "search",
            "api",
            "success",
            Duration::from_millis(42),
            Some("rows=10 total_matches=100"),
        );
        assert_eq!(
            msg,
            "event=search phase=api state=success elapsed_ms=42.0 rows=10 total_matches=100"
        );
    }
}
