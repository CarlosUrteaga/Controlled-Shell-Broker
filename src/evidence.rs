use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{ExecutionEvidence, ExecutionResponse, PolicyAudit};

const APP_DIR_NAME: &str = "llm-shell";
const RETENTION_DAYS_ENV: &str = "LLM_SHELL_EVIDENCE_RETENTION_DAYS";
const DEFAULT_RETENTION_DAYS: i64 = 30;
const MILLIS_PER_DAY: u128 = 86_400_000;

pub fn persist(response: &ExecutionResponse, policy_audit: &PolicyAudit) -> io::Result<PathBuf> {
    let state_dir = default_state_dir()?;
    persist_in_root(response, policy_audit, &state_dir)
}

pub fn default_state_dir() -> io::Result<PathBuf> {
    resolve_state_dir_for(std::env::consts::OS, |key| {
        std::env::var_os(key).map(PathBuf::from)
    })
}

pub fn persist_in_root(
    response: &ExecutionResponse,
    policy_audit: &PolicyAudit,
    state_dir: &Path,
) -> io::Result<PathBuf> {
    let timestamp = unix_timestamp_millis();
    let retention_days = retention_days()?;

    persist_in_root_at(response, policy_audit, state_dir, timestamp, retention_days)
}

fn persist_in_root_at(
    response: &ExecutionResponse,
    policy_audit: &PolicyAudit,
    state_dir: &Path,
    timestamp: u128,
    retention_days: i64,
) -> io::Result<PathBuf> {
    let evidence_root = state_dir.join("evidence");
    let evidence_dir = evidence_root.join(timestamp_millis_to_utc_date(timestamp));
    fs::create_dir_all(&evidence_dir)?;

    let file_name = format!("{}_{}.json", timestamp, response.request_id);
    let path = evidence_dir.join(file_name);
    let evidence = build_evidence(response, policy_audit, timestamp);

    fs::write(&path, evidence.to_json())?;
    cleanup_expired_evidence(
        &evidence_root,
        timestamp_days_since_epoch(timestamp),
        retention_days,
    )?;

    Ok(path)
}

fn retention_days() -> io::Result<i64> {
    retention_days_from_env(|key| std::env::var_os(key).map(PathBuf::from))
}

fn retention_days_from_env<F>(mut env: F) -> io::Result<i64>
where
    F: FnMut(&str) -> Option<PathBuf>,
{
    let Some(value) = env(RETENTION_DAYS_ENV) else {
        return Ok(DEFAULT_RETENTION_DAYS);
    };

    if value.as_os_str().is_empty() {
        return Err(invalid_retention_days("value must not be empty"));
    }

    let raw = value
        .to_str()
        .ok_or_else(|| invalid_retention_days("value must be valid UTF-8"))?;

    let parsed = raw
        .parse::<i64>()
        .map_err(|_| invalid_retention_days("value must be a positive integer"))?;

    if parsed <= 0 {
        return Err(invalid_retention_days("value must be greater than zero"));
    }

    Ok(parsed)
}

fn invalid_retention_days(reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("{RETENTION_DAYS_ENV} is invalid: {reason}"),
    )
}

fn cleanup_expired_evidence(
    evidence_root: &Path,
    current_day: i64,
    retention_days: i64,
) -> io::Result<()> {
    cleanup_expired_evidence_with_remover(evidence_root, current_day, retention_days, |path| {
        fs::remove_dir_all(path)
    })
}

fn cleanup_expired_evidence_with_remover<F>(
    evidence_root: &Path,
    current_day: i64,
    retention_days: i64,
    mut remove_dir_all: F,
) -> io::Result<()>
where
    F: FnMut(&Path) -> io::Result<()>,
{
    for entry in fs::read_dir(evidence_root)? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };

        let Some(bucket_day) = parse_date_to_days(&name) else {
            continue;
        };

        if current_day - bucket_day >= retention_days {
            match remove_dir_all(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
    }

    Ok(())
}

fn resolve_state_dir_for<F>(os: &str, mut env: F) -> io::Result<PathBuf>
where
    F: FnMut(&str) -> Option<PathBuf>,
{
    match os {
        "macos" => require_env_path(&mut env, "HOME").map(|home| {
            home.join("Library")
                .join("Application Support")
                .join(APP_DIR_NAME)
        }),
        "windows" => env("LOCALAPPDATA")
            .filter(|path| !path.as_os_str().is_empty())
            .or_else(|| env("APPDATA").filter(|path| !path.as_os_str().is_empty()))
            .map(|root| root.join(APP_DIR_NAME))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing LOCALAPPDATA/APPDATA")),
        _ => {
            if let Some(xdg_state_home) = env("XDG_STATE_HOME")
                .filter(|path| !path.as_os_str().is_empty())
                .filter(|path| path.is_absolute())
            {
                return Ok(xdg_state_home.join(APP_DIR_NAME));
            }

            require_env_path(&mut env, "HOME")
                .map(|home| home.join(".local").join("state").join(APP_DIR_NAME))
        }
    }
}

fn require_env_path<F>(env: &mut F, key: &str) -> io::Result<PathBuf>
where
    F: FnMut(&str) -> Option<PathBuf>,
{
    env(key)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("missing {key}")))
}

fn build_evidence(
    response: &ExecutionResponse,
    policy_audit: &PolicyAudit,
    timestamp: u128,
) -> ExecutionEvidence {
    ExecutionEvidence {
        event_type: if policy_audit.decision == "deny" {
            "execution.denied".to_string()
        } else {
            "execution.completed".to_string()
        },
        request_id: response.request_id.clone(),
        operation: response.operation.clone(),
        cwd: response.cwd.clone(),
        command: response.command.clone(),
        status: response.status.clone(),
        exit_code: response.exit_code,
        duration_ms: response.duration_ms,
        timed_out: response.timed_out,
        timestamp: timestamp.to_string(),
        error_code: response.error.as_ref().map(|error| error.code.clone()),
        policy_decision: policy_audit.decision.clone(),
        policy_reason: policy_audit.reason.clone(),
        inspection_category: inspection_category(&response.command).map(str::to_string),
        inspection_arg_count: response.command.len(),
    }
}

fn inspection_category(command: &[String]) -> Option<&'static str> {
    let command_0 = command.first()?;
    let basename = Path::new(command_0).file_name()?.to_str()?;

    match basename {
        "ls" => Some("list_paths"),
        "find" | "fd" => Some("find_files"),
        "rg" | "grep" => Some("search_text"),
        "cat" | "head" | "tail" | "sed" => Some("read_file"),
        _ => None,
    }
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn timestamp_days_since_epoch(timestamp_ms: u128) -> i64 {
    i64::try_from(timestamp_ms / MILLIS_PER_DAY).unwrap_or(i64::MAX)
}

fn timestamp_millis_to_utc_date(timestamp_ms: u128) -> String {
    let (year, month, day) = civil_from_days(timestamp_days_since_epoch(timestamp_ms));

    format!("{year:04}-{month:02}-{day:02}")
}

fn parse_date_to_days(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || !bytes[0..4].iter().all(u8::is_ascii_digit)
        || bytes[4] != b'-'
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || bytes[7] != b'-'
        || !bytes[8..10].iter().all(u8::is_ascii_digit)
    {
        return None;
    }

    let year = value[0..4].parse::<i32>().ok()?;
    let month = value[5..7].parse::<u32>().ok()?;
    let day = value[8..10].parse::<u32>().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let days = days_from_civil(year, month, day);
    let (round_trip_year, round_trip_month, round_trip_day) = civil_from_days(days);

    (year == round_trip_year && month == round_trip_month && day == round_trip_day).then_some(days)
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year + i32::from(month <= 2);

    (year, month, day)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = (year - era * 400) as u32;
    let month_prime = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    era as i64 * 146_097 + doe as i64 - 719_468
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::types::{ErrorDetail, ExecutionResponse, PolicyAudit};

    fn response() -> ExecutionResponse {
        ExecutionResponse {
            request_id: "req-evidence-test".to_string(),
            operation: "run".to_string(),
            status: "success".to_string(),
            cwd: PathBuf::from(".")
                .canonicalize()
                .expect("current directory should resolve"),
            command: vec!["echo".to_string(), "hello".to_string()],
            exit_code: Some(0),
            stdout: "hello\n".to_string(),
            stderr: "debug\n".to_string(),
            duration_ms: 5,
            timed_out: false,
            error: None,
        }
    }

    fn denied_response() -> ExecutionResponse {
        ExecutionResponse {
            request_id: "req-denied-test".to_string(),
            operation: "run".to_string(),
            status: "denied".to_string(),
            cwd: PathBuf::from(".")
                .canonicalize()
                .expect("current directory should resolve"),
            command: vec!["rm".to_string(), "-rf".to_string(), ".".to_string()],
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            timed_out: false,
            error: Some(ErrorDetail {
                code: "denied_executable".to_string(),
                message: "The request executable is denied by broker policy.".to_string(),
            }),
        }
    }

    fn response_with_command(command: Vec<String>) -> ExecutionResponse {
        ExecutionResponse {
            command,
            ..response()
        }
    }

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "llm-shell-evidence-{name}-{}-{}",
            std::process::id(),
            unix_timestamp_millis()
        ))
    }

    #[test]
    fn persists_metadata_only_evidence_in_dated_directory() {
        let temp_root = temp_root("metadata");
        let response = response();
        let timestamp = days_from_civil(2026, 5, 27) as u128 * MILLIS_PER_DAY;

        let path = persist_in_root_at(&response, &PolicyAudit::allow(), &temp_root, timestamp, 30)
            .expect("evidence should persist");
        let contents = fs::read_to_string(&path).expect("evidence file should be readable");

        assert!(path.starts_with(temp_root.join("evidence").join("2026-05-27")));
        assert!(path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_req-evidence-test.json")));
        assert!(contents.contains("\"request_id\":\"req-evidence-test\""));
        assert!(contents.contains("\"timestamp\":\""));
        assert!(contents.contains("\"policy_decision\":\"allow\""));
        assert!(contents.contains("\"inspection_category\":null"));
        assert!(contents.contains("\"inspection_arg_count\":2"));
        assert!(!contents.contains("stdout"));
        assert!(!contents.contains("stderr"));

        fs::remove_file(&path).expect("evidence file cleanup should succeed");
        fs::remove_dir_all(&temp_root).expect("evidence directory cleanup should succeed");
    }

    #[test]
    fn persists_denied_evidence_with_policy_reason() {
        let temp_root = temp_root("denied");
        let response = denied_response();
        let timestamp = days_from_civil(2026, 5, 27) as u128 * MILLIS_PER_DAY;

        let path = persist_in_root_at(
            &response,
            &PolicyAudit::deny("denied_executable".to_string()),
            &temp_root,
            timestamp,
            30,
        )
        .expect("denied evidence should persist");
        let contents = fs::read_to_string(&path).expect("evidence file should be readable");

        assert!(contents.contains("\"event_type\":\"execution.denied\""));
        assert!(contents.contains("\"status\":\"denied\""));
        assert!(contents.contains("\"policy_decision\":\"deny\""));
        assert!(contents.contains("\"policy_reason\":\"denied_executable\""));
        assert!(contents.contains("\"inspection_category\":null"));
        assert!(contents.contains("\"inspection_arg_count\":3"));
        assert!(!contents.contains("stdout"));
        assert!(!contents.contains("stderr"));

        fs::remove_file(&path).expect("evidence file cleanup should succeed");
        fs::remove_dir_all(&temp_root).expect("evidence directory cleanup should succeed");
    }

    #[test]
    fn persists_inspection_metrics_for_repository_search_command() {
        let temp_root = temp_root("inspection");
        let response = response_with_command(vec![
            "rg".to_string(),
            "Phase 3B".to_string(),
            "docs".to_string(),
        ]);
        let timestamp = days_from_civil(2026, 5, 27) as u128 * MILLIS_PER_DAY;

        let path = persist_in_root_at(&response, &PolicyAudit::allow(), &temp_root, timestamp, 30)
            .expect("inspection evidence should persist");
        let contents = fs::read_to_string(&path).expect("evidence file should be readable");

        assert!(contents.contains("\"inspection_category\":\"search_text\""));
        assert!(contents.contains("\"inspection_arg_count\":3"));
        assert!(!contents.contains("stdout"));
        assert!(!contents.contains("stderr"));

        fs::remove_file(&path).expect("evidence file cleanup should succeed");
        fs::remove_dir_all(&temp_root).expect("evidence directory cleanup should succeed");
    }

    #[test]
    fn resolve_state_dir_uses_absolute_xdg_state_home_on_unix() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "XDG_STATE_HOME" => Some(PathBuf::from("/xdg-state")),
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(resolved, PathBuf::from("/xdg-state").join(APP_DIR_NAME));
    }

    #[test]
    fn resolve_state_dir_falls_back_to_home_on_unix_when_xdg_missing() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/home/tester")
                .join(".local")
                .join("state")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_ignores_relative_xdg_state_home() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "XDG_STATE_HOME" => Some(PathBuf::from("relative/state")),
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/home/tester")
                .join(".local")
                .join("state")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_ignores_empty_xdg_state_home() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "XDG_STATE_HOME" => Some(PathBuf::new()),
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/home/tester")
                .join(".local")
                .join("state")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_uses_application_support_on_macos() {
        let resolved = resolve_state_dir_for("macos", |key| match key {
            "HOME" => Some(PathBuf::from("/Users/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/Users/tester")
                .join("Library")
                .join("Application Support")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_errors_when_home_missing_on_macos() {
        let error =
            resolve_state_dir_for("macos", |_key| None).expect_err("missing HOME should error");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn resolve_state_dir_prefers_localappdata_on_windows() {
        let resolved = resolve_state_dir_for("windows", |key| match key {
            "LOCALAPPDATA" => Some(PathBuf::from(r"C:\Users\tester\AppData\Local")),
            "APPDATA" => Some(PathBuf::from(r"C:\Users\tester\AppData\Roaming")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from(r"C:\Users\tester\AppData\Local").join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_falls_back_to_appdata_on_windows() {
        let resolved = resolve_state_dir_for("windows", |key| match key {
            "APPDATA" => Some(PathBuf::from(r"C:\Users\tester\AppData\Roaming")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from(r"C:\Users\tester\AppData\Roaming").join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_errors_when_windows_env_missing() {
        let error = resolve_state_dir_for("windows", |_key| None)
            .expect_err("missing windows env should error");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn default_state_dir_is_not_the_v0_temp_directory_baseline() {
        let resolved = default_state_dir().expect("default state dir should resolve");

        assert_ne!(resolved, std::env::temp_dir().join(APP_DIR_NAME));
        assert!(resolved.ends_with(APP_DIR_NAME));
    }

    #[test]
    fn retention_days_defaults_to_thirty_when_env_is_unset() {
        let resolved = retention_days_from_env(|_key| None).expect("default should resolve");

        assert_eq!(resolved, 30);
    }

    #[test]
    fn retention_days_accepts_positive_env_override() {
        let resolved = retention_days_from_env(|key| match key {
            RETENTION_DAYS_ENV => Some(PathBuf::from("14")),
            _ => None,
        })
        .expect("override should resolve");

        assert_eq!(resolved, 14);
    }

    #[test]
    fn retention_days_rejects_invalid_env_values() {
        for value in ["", "0", "-1", "abc"] {
            let error = retention_days_from_env(|key| match key {
                RETENTION_DAYS_ENV => Some(PathBuf::from(value)),
                _ => None,
            })
            .expect_err("invalid retention should error");

            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn timestamp_millis_to_utc_date_handles_leap_year_boundaries() {
        assert_eq!(
            timestamp_millis_to_utc_date(days_from_civil(2024, 2, 28) as u128 * MILLIS_PER_DAY),
            "2024-02-28"
        );
        assert_eq!(
            timestamp_millis_to_utc_date(days_from_civil(2024, 2, 29) as u128 * MILLIS_PER_DAY),
            "2024-02-29"
        );
        assert_eq!(
            timestamp_millis_to_utc_date(days_from_civil(2024, 3, 1) as u128 * MILLIS_PER_DAY),
            "2024-03-01"
        );
        assert_eq!(
            timestamp_millis_to_utc_date(days_from_civil(2023, 3, 1) as u128 * MILLIS_PER_DAY),
            "2023-03-01"
        );
    }

    #[test]
    fn parse_date_to_days_validates_real_utc_dates() {
        assert_eq!(parse_date_to_days("1970-01-01"), Some(0));
        assert_eq!(
            parse_date_to_days("2024-02-29"),
            Some(days_from_civil(2024, 2, 29))
        );
        assert_eq!(parse_date_to_days("2023-02-29"), None);
        assert_eq!(parse_date_to_days("2026-05-2a"), None);
        assert_eq!(parse_date_to_days("evidence-old"), None);
    }

    #[test]
    fn cleanup_removes_only_expired_date_buckets() {
        let root = temp_root("cleanup").join("evidence");
        let expired = root.join("2026-04-27");
        let recent = root.join("2026-04-28");
        let current = root.join("2026-05-27");
        let malformed = root.join("2026-05-2a");
        let non_date = root.join("evidence-old");

        for path in [&expired, &recent, &current, &malformed, &non_date] {
            fs::create_dir_all(path).expect("test evidence directory should be created");
            fs::write(path.join("event.json"), "{}").expect("test evidence file should be created");
        }

        cleanup_expired_evidence(&root, days_from_civil(2026, 5, 27), 30)
            .expect("cleanup should succeed");

        assert!(!expired.exists());
        assert!(recent.exists());
        assert!(current.exists());
        assert!(malformed.exists());
        assert!(non_date.exists());

        fs::remove_dir_all(root.parent().expect("root should have parent"))
            .expect("test cleanup should succeed");
    }

    #[test]
    fn cleanup_ignores_non_directory_date_entries() {
        let root = temp_root("cleanup-file").join("evidence");
        fs::create_dir_all(&root).expect("test evidence root should be created");
        fs::write(root.join("2026-04-27"), "{}").expect("date-named file should be created");

        cleanup_expired_evidence(&root, days_from_civil(2026, 5, 27), 30)
            .expect("cleanup should succeed");

        assert!(root.join("2026-04-27").is_file());

        fs::remove_dir_all(root.parent().expect("root should have parent"))
            .expect("test cleanup should succeed");
    }

    #[test]
    fn cleanup_tolerates_not_found_when_parallel_run_removed_bucket() {
        let root = temp_root("cleanup-not-found").join("evidence");
        let expired = root.join("2026-04-27");
        fs::create_dir_all(&expired).expect("expired directory should be created");

        cleanup_expired_evidence_with_remover(&root, days_from_civil(2026, 5, 27), 30, |_path| {
            Err(io::Error::new(io::ErrorKind::NotFound, "already removed"))
        })
        .expect("not found should be tolerated");

        fs::remove_dir_all(root.parent().expect("root should have parent"))
            .expect("test cleanup should succeed");
    }

    #[test]
    fn cleanup_returns_other_remove_errors() {
        let root = temp_root("cleanup-error").join("evidence");
        let expired = root.join("2026-04-27");
        fs::create_dir_all(&expired).expect("expired directory should be created");

        let error = cleanup_expired_evidence_with_remover(
            &root,
            days_from_civil(2026, 5, 27),
            30,
            |_path| Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
        )
        .expect_err("cleanup error should propagate");

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);

        fs::remove_dir_all(root.parent().expect("root should have parent"))
            .expect("test cleanup should succeed");
    }

    #[test]
    fn classifies_inspection_discovery_commands() {
        assert_eq!(
            inspection_category(&["ls".to_string(), "src".to_string()]),
            Some("list_paths")
        );
        assert_eq!(
            inspection_category(&["/usr/bin/find".to_string(), ".".to_string()]),
            Some("find_files")
        );
        assert_eq!(
            inspection_category(&["rg".to_string(), "Phase 3B".to_string()]),
            Some("search_text")
        );
        assert_eq!(
            inspection_category(&["head".to_string(), "README.md".to_string()]),
            Some("read_file")
        );
    }

    #[test]
    fn leaves_non_inspection_commands_uncategorized() {
        assert_eq!(
            inspection_category(&["cargo".to_string(), "test".to_string()]),
            None
        );
        assert_eq!(inspection_category(&[]), None);
    }
}
