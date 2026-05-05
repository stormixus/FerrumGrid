//! Automation scheduler runner — Schedule::Interval / Once 자동 발사.
//!
//! Plan v7 Phase 4b3 deferred-runner. UI thread 와 공유되는
//! `Arc<RwLock<AutomationStore>>` 를 1초 tick 으로 관찰해서 due task 를
//! `DbCommand::ExecuteAutomation` 으로 발사. 응답 (`AutomationResult`) 은
//! UI thread 의 app.rs handler 가 store.mark_run 으로 적용.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use chrono::Utc;
use tokio::sync::mpsc;

use crate::automation::scheduler::AutomationStore;
use crate::db::bridge::DbCommand;
use crate::types::ConnectionId;

/// 1초 tick 단위. 단위 테스트는 더 짧은 값으로 호출 가능.
pub const TICK_INTERVAL: Duration = Duration::from_secs(1);

/// 단일 tick 처리 — store 에서 due task 찾아 cmd_tx 로 발사.
///
/// 반환: 발사된 task 수.
///
/// 락 정책: store.read() 만 사용 (find_due_tasks 는 read-only). mark_run 은
/// AutomationResult 응답 수신 시 UI thread 가 호출.
pub fn tick_once(
    store: &Arc<RwLock<AutomationStore>>,
    cmd_tx: &mpsc::Sender<DbCommand>,
    conn_id: ConnectionId,
) -> usize {
    let now = Utc::now();
    let due = match store.read() {
        Ok(s) => s.find_due_tasks(now),
        Err(poisoned) => {
            tracing::error!("automation store lock poisoned: {poisoned}");
            return 0;
        }
    };
    let count = due.len();
    for (task_id, sql) in due {
        let cmd = DbCommand::ExecuteAutomation {
            conn_id,
            task_id,
            sql,
        };
        if let Err(e) = cmd_tx.try_send(cmd) {
            tracing::warn!("automation: failed to dispatch task {task_id}: {e}");
        }
    }
    count
}

/// 무한 루프 runner — `tokio::spawn` 으로 띄움. 종료는 `shutdown_rx` 신호.
pub async fn run_scheduler(
    store: Arc<RwLock<AutomationStore>>,
    cmd_tx: mpsc::Sender<DbCommand>,
    conn_id: ConnectionId,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let mut ticker = tokio::time::interval(TICK_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let fired = tick_once(&store, &cmd_tx, conn_id);
                if fired > 0 {
                    tracing::debug!("automation: dispatched {fired} task(s)");
                }
            }
            _ = &mut shutdown_rx => {
                tracing::info!("automation runner shutting down");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::scheduler::{ApplyResult, Schedule, ScheduledTask};
    use std::time::Duration as StdDuration;

    fn make_store_with_due_interval_task() -> Arc<RwLock<AutomationStore>> {
        let mut store = AutomationStore::default();
        // Interval 의 첫 next_run 은 now (compute_next_run rules) — 항상 due.
        store.add(ScheduledTask::new(
            "test interval",
            "SELECT 1",
            Schedule::Interval {
                period: StdDuration::from_secs(60),
            },
        ));
        Arc::new(RwLock::new(store))
    }

    fn fake_conn_id() -> ConnectionId {
        ConnectionId(uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn tick_once_fires_due_tasks() {
        let store = make_store_with_due_interval_task();
        let (tx, mut rx) = mpsc::channel::<DbCommand>(8);
        let count = tick_once(&store, &tx, fake_conn_id());
        assert_eq!(count, 1);
        let cmd = rx.try_recv().expect("expected ExecuteAutomation");
        assert!(matches!(cmd, DbCommand::ExecuteAutomation { .. }));
    }

    #[tokio::test]
    async fn tick_once_with_empty_store_fires_nothing() {
        let store = Arc::new(RwLock::new(AutomationStore::default()));
        let (tx, mut rx) = mpsc::channel::<DbCommand>(8);
        let count = tick_once(&store, &tx, fake_conn_id());
        assert_eq!(count, 0);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn tick_once_with_no_due_task_fires_nothing() {
        let mut store = AutomationStore::default();
        // Once 미래 시각 — 아직 due 아님
        store.add(ScheduledTask::new(
            "future once",
            "SELECT 2",
            Schedule::Once {
                at: Utc::now() + chrono::Duration::hours(1),
            },
        ));
        let store = Arc::new(RwLock::new(store));
        let (tx, _rx) = mpsc::channel::<DbCommand>(8);
        let count = tick_once(&store, &tx, fake_conn_id());
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn mark_run_after_once_makes_next_run_none() {
        let mut store = AutomationStore::default();
        let task_id = store.add(ScheduledTask::new(
            "once",
            "SELECT 3",
            Schedule::Once { at: Utc::now() },
        ));
        let store = Arc::new(RwLock::new(store));

        // First tick fires
        let (tx, mut rx) = mpsc::channel::<DbCommand>(8);
        let count = tick_once(&store, &tx, fake_conn_id());
        assert_eq!(count, 1);
        let _ = rx.try_recv();

        // Apply mark_run (simulating AutomationResult handler)
        store.write().unwrap().mark_run(
            task_id,
            Utc::now(),
            ApplyResult::Success { rows_affected: 0 },
        );

        // Second tick — Once should NOT fire again
        let count2 = tick_once(&store, &tx, fake_conn_id());
        assert_eq!(count2, 0, "Once 가 1회 실행 후 next_run=None 이어야 함");
    }

    #[tokio::test]
    async fn shared_arc_lock_allows_concurrent_read_iter() {
        // UI thread + runner 동시 read 가 deadlock 없이 가능해야 함
        let store = make_store_with_due_interval_task();
        let store_clone = store.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let s = store_clone.read().unwrap();
            s.iter().count()
        });
        let direct_count = store.read().unwrap().iter().count();
        let bg_count = handle.await.unwrap();
        assert_eq!(direct_count, 1);
        assert_eq!(bg_count, 1);
    }

    #[tokio::test]
    async fn shutdown_signal_makes_run_scheduler_return() {
        // run_scheduler 가 shutdown_rx 신호 수신 시 break 하여 종료해야 함.
        let store = Arc::new(RwLock::new(AutomationStore::default()));
        let (cmd_tx, _cmd_rx) = mpsc::channel::<DbCommand>(8);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let scheduler = tokio::spawn(run_scheduler(
            store,
            cmd_tx,
            fake_conn_id(),
            shutdown_rx,
        ));
        // 즉시 shutdown 발사 — ticker 첫 tick 대기 (1초) 와 race
        let _ = shutdown_tx.send(());
        // 1.5초 timeout 으로 join (ticker 1초 + 여유)
        let result = tokio::time::timeout(StdDuration::from_millis(1500), scheduler).await;
        assert!(result.is_ok(), "run_scheduler did not exit within 1.5s");
        assert!(result.unwrap().is_ok(), "run_scheduler panicked");
    }
}
