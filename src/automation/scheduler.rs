//! Automation task types + scheduling 결정 로직.
//!
//! Plan v7 Phase 4b1 — backend infra (no runtime spawn). Runtime scheduler
//! (tokio::spawn + interval) 는 Phase 4b3 (`runner.rs`) 에서 도입.
//! Phase 4b3 추가: `AutomationStore` (HashMap-based registry) + `find_due_tasks`
//! / `mark_run` 헬퍼.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 예약 정책. Plan v7 §6 Phase 4b — *최소* 2 종 (즉시 실행 + 주기 실행).
/// Cron 스타일 (특정 시각/요일) 은 Phase 4b 의 minimum acceptance 외 — 향후 확장.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schedule {
    /// 단일 시점 실행 (해당 시각 도달 시 1회).
    Once { at: DateTime<Utc> },
    /// 주기 실행 (last_run + period 도달 시 다시).
    Interval { period: Duration },
}

/// SQL 실행 결과 (응답으로 노출).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ApplyResult {
    Success { rows_affected: u64 },
    Failed { error: String },
}

#[allow(dead_code)]
impl ApplyResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ApplyResult::Success { .. })
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            ApplyResult::Success { .. } => "Success",
            ApplyResult::Failed { .. } => "Failed",
        }
    }
}

/// 단일 자동화 작업.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub title: String,
    pub sql: String,
    pub schedule: Schedule,
    /// 마지막 실행 시각 (성공/실패 무관). `None` 이면 미실행.
    pub last_run: Option<DateTime<Utc>>,
    /// 다음 실행 예정 시각. `compute_next_run` 결과 캐시 — `None` 은 더 이상 실행
    /// 안 함 (Once 의 경우 1회 실행 후 None).
    pub next_run: Option<DateTime<Utc>>,
    /// 마지막 실행 결과 (UI 표시 용).
    pub last_result: Option<ApplyResult>,
}

#[allow(dead_code)]
impl ScheduledTask {
    /// 새 task 생성. `next_run` 은 schedule 에서 즉시 derive.
    pub fn new(title: impl Into<String>, sql: impl Into<String>, schedule: Schedule) -> Self {
        let now = Utc::now();
        let next_run = compute_next_run(now, &schedule, None);
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            sql: sql.into(),
            schedule,
            last_run: None,
            next_run,
            last_result: None,
        }
    }
}

/// 다음 실행 시각 계산.
///
/// - `Once { at }` + `last_run.is_none()` → `Some(at)` (한 번만)
/// - `Once { at }` + `last_run.is_some()` → `None` (이미 실행됨)
/// - `Interval { period }` + `last_run.is_none()` → `Some(now)` (즉시 첫 실행)
/// - `Interval { period }` + `last_run.is_some()` → `Some(last + period)`
#[allow(dead_code)]
pub fn compute_next_run(
    now: DateTime<Utc>,
    schedule: &Schedule,
    last_run: Option<DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
    match schedule {
        Schedule::Once { at } => {
            if last_run.is_some() {
                None
            } else {
                Some(*at)
            }
        }
        Schedule::Interval { period } => match last_run {
            None => Some(now),
            Some(last) => {
                let delta = chrono::Duration::from_std(*period).ok()?;
                Some(last + delta)
            }
        },
    }
}

/// 현재 시각에 task 가 실행 가능한지 여부.
#[allow(dead_code)]
pub fn is_due(task: &ScheduledTask, now: DateTime<Utc>) -> bool {
    match task.next_run {
        Some(next) => now >= next,
        None => false,
    }
}

/// Plan v7 Phase 4b3 — task registry. `Arc<RwLock<AutomationStore>>` 로
/// `runner.rs` 의 background loop 와 UI thread 가 공유.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct AutomationStore {
    tasks: HashMap<Uuid, ScheduledTask>,
}

#[allow(dead_code)]
impl AutomationStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// task 등록 — `next_run` 은 이미 `ScheduledTask::new` 에서 derived.
    pub fn add(&mut self, task: ScheduledTask) -> Uuid {
        let id = task.id;
        self.tasks.insert(id, task);
        id
    }

    /// task 제거 (UI 의 Delete 액션).
    pub fn remove(&mut self, id: Uuid) -> Option<ScheduledTask> {
        self.tasks.remove(&id)
    }

    /// 등록된 모든 task (UI 표시 용).
    pub fn iter(&self) -> impl Iterator<Item = &ScheduledTask> {
        self.tasks.values()
    }

    /// task 개수.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// 현재 시각에 due 인 task 들 (background runner 가 1s tick 마다 호출).
    /// 결과는 (id, sql) 튜플 — runner 가 bridge 로 dispatch 후 `mark_run` 호출.
    pub fn find_due_tasks(&self, now: DateTime<Utc>) -> Vec<(Uuid, String)> {
        self.tasks
            .values()
            .filter(|task| is_due(task, now))
            .map(|task| (task.id, task.sql.clone()))
            .collect()
    }

    /// task 실행 후 last_run / last_result / next_run 갱신.
    /// `result` 는 `runner` 가 bridge 응답 (`DbResponse::AutomationResult`) 에서
    /// 회수하여 전달.
    pub fn mark_run(&mut self, id: Uuid, now: DateTime<Utc>, result: ApplyResult) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.last_run = Some(now);
            task.last_result = Some(result);
            task.next_run = compute_next_run(now, &task.schedule, Some(now));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .expect("valid rfc3339")
            .with_timezone(&Utc)
    }

    #[test]
    fn once_schedules_to_at_when_unrun() {
        let now = ts("2026-05-05T01:00:00Z");
        let at = ts("2026-05-05T03:00:00Z");
        let next = compute_next_run(now, &Schedule::Once { at }, None);
        assert_eq!(next, Some(at));
    }

    #[test]
    fn once_returns_none_after_first_run() {
        let now = ts("2026-05-05T01:00:00Z");
        let at = ts("2026-05-05T03:00:00Z");
        let last = ts("2026-05-05T03:00:01Z");
        let next = compute_next_run(now, &Schedule::Once { at }, Some(last));
        assert_eq!(next, None);
    }

    #[test]
    fn interval_first_run_is_now() {
        let now = ts("2026-05-05T01:00:00Z");
        let next = compute_next_run(
            now,
            &Schedule::Interval {
                period: Duration::from_secs(3600),
            },
            None,
        );
        assert_eq!(next, Some(now));
    }

    #[test]
    fn interval_next_is_last_plus_period() {
        let now = ts("2026-05-05T02:30:00Z");
        let last = ts("2026-05-05T01:00:00Z");
        let next = compute_next_run(
            now,
            &Schedule::Interval {
                period: Duration::from_secs(3600),
            },
            Some(last),
        );
        // last + 1h = 02:00 (regardless of now).
        assert_eq!(next, Some(ts("2026-05-05T02:00:00Z")));
    }

    #[test]
    fn is_due_false_when_next_is_in_future() {
        let now = ts("2026-05-05T01:00:00Z");
        let task = ScheduledTask {
            id: Uuid::nil(),
            title: "x".to_string(),
            sql: "SELECT 1".to_string(),
            schedule: Schedule::Interval {
                period: Duration::from_secs(60),
            },
            last_run: None,
            next_run: Some(ts("2026-05-05T01:05:00Z")),
            last_result: None,
        };
        assert!(!is_due(&task, now));
    }

    #[test]
    fn is_due_true_when_next_equal_or_before_now() {
        let now = ts("2026-05-05T01:05:00Z");
        let mut task = ScheduledTask {
            id: Uuid::nil(),
            title: "x".to_string(),
            sql: "SELECT 1".to_string(),
            schedule: Schedule::Interval {
                period: Duration::from_secs(60),
            },
            last_run: None,
            next_run: Some(ts("2026-05-05T01:05:00Z")),
            last_result: None,
        };
        assert!(is_due(&task, now), "exact equality should be due");

        // Past next_run also due
        task.next_run = Some(ts("2026-05-05T00:00:00Z"));
        assert!(is_due(&task, now));
    }

    #[test]
    fn is_due_false_when_next_run_is_none() {
        let task = ScheduledTask {
            id: Uuid::nil(),
            title: "x".to_string(),
            sql: "SELECT 1".to_string(),
            schedule: Schedule::Once {
                at: ts("2026-01-01T00:00:00Z"),
            },
            last_run: Some(ts("2026-01-01T00:00:01Z")),
            next_run: None,
            last_result: None,
        };
        assert!(!is_due(&task, ts("2026-12-31T23:59:59Z")));
    }

    #[test]
    fn apply_result_success_label() {
        let r = ApplyResult::Success { rows_affected: 42 };
        assert!(r.is_success());
        assert_eq!(r.short_label(), "Success");
    }

    #[test]
    fn apply_result_failed_label() {
        let r = ApplyResult::Failed {
            error: "oops".to_string(),
        };
        assert!(!r.is_success());
        assert_eq!(r.short_label(), "Failed");
    }

    #[test]
    fn scheduled_task_new_sets_initial_next_run() {
        let task = ScheduledTask::new(
            "vacuum",
            "VACUUM FULL",
            Schedule::Interval {
                period: Duration::from_secs(86400),
            },
        );
        assert!(task.next_run.is_some());
        assert!(task.last_run.is_none());
        assert!(task.last_result.is_none());
    }

    // ---------- AutomationStore ----------

    #[test]
    fn store_add_increments_len() {
        let mut store = AutomationStore::new();
        assert!(store.is_empty());
        let task = ScheduledTask::new("t", "SELECT 1", Schedule::Once {
            at: ts("2026-05-05T03:00:00Z"),
        });
        store.add(task);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    #[test]
    fn store_remove_returns_task() {
        let mut store = AutomationStore::new();
        let task = ScheduledTask::new("t", "SELECT 1", Schedule::Interval {
            period: Duration::from_secs(60),
        });
        let id = store.add(task);
        let removed = store.remove(id);
        assert!(removed.is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn store_find_due_tasks_returns_only_due_ones() {
        let mut store = AutomationStore::new();
        let mut due_task = ScheduledTask::new(
            "due",
            "SELECT 1",
            Schedule::Once {
                at: ts("2026-05-05T01:00:00Z"),
            },
        );
        due_task.next_run = Some(ts("2026-05-05T01:00:00Z"));
        let mut not_due_task = ScheduledTask::new(
            "future",
            "SELECT 2",
            Schedule::Once {
                at: ts("2026-12-31T23:59:59Z"),
            },
        );
        not_due_task.next_run = Some(ts("2026-12-31T23:59:59Z"));

        store.add(due_task);
        store.add(not_due_task);

        let now = ts("2026-05-05T02:00:00Z");
        let due = store.find_due_tasks(now);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].1, "SELECT 1");
    }

    #[test]
    fn store_mark_run_updates_state_and_recomputes_next() {
        let mut store = AutomationStore::new();
        let task = ScheduledTask::new(
            "interval",
            "SELECT 1",
            Schedule::Interval {
                period: Duration::from_secs(3600),
            },
        );
        let id = store.add(task);

        let run_at = ts("2026-05-05T01:00:00Z");
        store.mark_run(
            id,
            run_at,
            ApplyResult::Success { rows_affected: 0 },
        );

        let task = store.tasks.get(&id).expect("still present");
        assert_eq!(task.last_run, Some(run_at));
        assert!(matches!(
            task.last_result,
            Some(ApplyResult::Success { rows_affected: 0 })
        ));
        // next_run = run_at + 1h
        assert_eq!(task.next_run, Some(ts("2026-05-05T02:00:00Z")));
    }

    #[test]
    fn store_mark_run_once_clears_next_run() {
        let mut store = AutomationStore::new();
        let task = ScheduledTask::new(
            "once",
            "SELECT 1",
            Schedule::Once {
                at: ts("2026-05-05T01:00:00Z"),
            },
        );
        let id = store.add(task);

        store.mark_run(
            id,
            ts("2026-05-05T01:00:01Z"),
            ApplyResult::Success { rows_affected: 1 },
        );

        let task = store.tasks.get(&id).expect("still present");
        assert!(task.last_run.is_some());
        assert!(
            task.next_run.is_none(),
            "Once task 가 mark_run 후 더 이상 실행 안 됨"
        );
    }

    #[test]
    fn store_iter_yields_all_tasks() {
        let mut store = AutomationStore::new();
        for i in 0..3 {
            store.add(ScheduledTask::new(
                format!("t{i}"),
                "SELECT 1",
                Schedule::Interval {
                    period: Duration::from_secs(60),
                },
            ));
        }
        let count = store.iter().count();
        assert_eq!(count, 3);
    }
}
