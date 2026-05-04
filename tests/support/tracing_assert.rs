//! Tracing assertion helper — predicate-based event verification.
//!
//! Plan v7 Phase 0 산출물. snapshot 비교의 brittleness (필드 순서/format 의존)
//! 를 회피하기 위해 `LayerCollector` 로 구조적으로 캡처한 이벤트를 필드 단위
//! predicate 로 검증한다.
//!
//! Usage (`with_capture` 권장):
//! ```ignore
//! #[path = "support/tracing_assert.rs"]
//! mod tracing_assert;
//! use tracing_assert::{assert_event, with_capture};
//!
//! let events = with_capture(|| {
//!     tracing::info!(target: "ferrumgrid::mutation", op = "insert", applied = 1, "applied");
//! });
//! assert_event(&events, "ferrumgrid::mutation")
//!     .field_eq("op", "insert")
//!     .field_present("applied");
//! ```
//!
//! 수동 lifecycle 이 필요할 때는 `capture()` 로 핸들을 받고 `events()` 로 회수한다.
//! 핸들이 drop 되기 전에 반드시 `events()` 를 호출해야 한다.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::subscriber::{set_default, DefaultGuard};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::Registry;
use tracing_subscriber::Layer;

/// 캡처된 단일 tracing event.
#[derive(Debug, Clone)]
pub struct CapturedEvent {
    pub target: String,
    pub level: tracing::Level,
    pub fields: BTreeMap<String, String>,
}

/// Layer 가 이벤트를 누적하는 공유 버퍼.
type Sink = Arc<Mutex<Vec<CapturedEvent>>>;

/// `tracing-subscriber` Layer 구현 — 모든 이벤트를 sink 에 누적한다.
///
/// `LayerCollector` alias 로도 노출 (Plan v7 §12 명세 명칭).
pub struct CollectorLayer {
    sink: Sink,
}

/// Plan v7 §12 명세 명칭 alias. 신규 코드는 이 이름을 우선 사용 권장.
#[allow(dead_code)]
pub type LayerCollector = CollectorLayer;

impl<S> Layer<S> for CollectorLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        let captured = CapturedEvent {
            target: event.metadata().target().to_string(),
            level: *event.metadata().level(),
            fields: visitor.fields,
        };
        if let Ok(mut sink) = self.sink.lock() {
            sink.push(captured);
        }
    }
}

#[derive(Default)]
struct FieldVisitor {
    fields: BTreeMap<String, String>,
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // `Debug` 출력의 따옴표 제거 (string 인 경우 `"foo"` → `foo`).
        let formatted = format!("{value:?}");
        let trimmed = formatted.trim_matches('"').to_string();
        self.fields.insert(field.name().to_string(), trimmed);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}

/// 캡처 핸들. drop 시 subscriber 가 해제된다.
pub struct CaptureHandle {
    sink: Sink,
    _guard: DefaultGuard,
}

impl CaptureHandle {
    /// 현재까지 캡처된 이벤트의 스냅샷을 반환한다.
    pub fn events(&self) -> Vec<CapturedEvent> {
        self.sink.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// 클로저 실행 동안의 모든 이벤트를 캡처한다.
///
/// 반환된 `Vec<CapturedEvent>` 는 클로저가 emit 한 모든 이벤트를 순서대로 담는다.
pub fn with_capture<F>(f: F) -> Vec<CapturedEvent>
where
    F: FnOnce(),
{
    let handle = capture();
    f();
    handle.events()
}

/// Subscriber 가 active 인 동안 이벤트를 누적하는 핸들을 만든다.
///
/// `CaptureHandle` 가 살아 있는 한 현재 thread 의 tracing 출력이 캡처된다.
pub fn capture() -> CaptureHandle {
    let sink: Sink = Arc::new(Mutex::new(Vec::new()));
    let layer = CollectorLayer { sink: sink.clone() };
    let subscriber = Registry::default().with(layer);
    let guard = set_default(subscriber);
    CaptureHandle {
        sink,
        _guard: guard,
    }
}

/// 특정 target 의 가장 최근 이벤트에 대한 assertion 빌더.
pub fn assert_event<'a>(events: &'a [CapturedEvent], target: &str) -> EventAssertion<'a> {
    let event = events
        .iter()
        .rev()
        .find(|e| e.target == target)
        .unwrap_or_else(|| {
            panic!(
                "no event captured with target '{target}' — captured targets: {:?}",
                events.iter().map(|e| &e.target).collect::<Vec<_>>()
            )
        });
    EventAssertion { event }
}

/// 필드 단위 predicate chain.
pub struct EventAssertion<'a> {
    event: &'a CapturedEvent,
}

impl<'a> EventAssertion<'a> {
    /// 필드가 존재하고 값이 비어있지 않은지 검증.
    pub fn field_non_empty(self, key: &str) -> Self {
        let value = self.event.fields.get(key).unwrap_or_else(|| {
            panic!(
                "field '{key}' missing in event (target={}, fields={:?})",
                self.event.target, self.event.fields
            )
        });
        assert!(
            !value.is_empty(),
            "field '{key}' is empty in event (target={})",
            self.event.target
        );
        self
    }

    /// 필드 값이 expected 와 일치하는지 검증.
    pub fn field_eq(self, key: &str, expected: &str) -> Self {
        let value = self.event.fields.get(key).unwrap_or_else(|| {
            panic!(
                "field '{key}' missing in event (target={}, fields={:?})",
                self.event.target, self.event.fields
            )
        });
        assert_eq!(
            value, expected,
            "field '{key}' value mismatch (target={})",
            self.event.target
        );
        self
    }

    /// 필드가 존재하기만 하면 통과 (값 무관).
    pub fn field_present(self, key: &str) -> Self {
        assert!(
            self.event.fields.contains_key(key),
            "field '{key}' missing in event (target={}, fields={:?})",
            self.event.target,
            self.event.fields
        );
        self
    }

    /// level 검증.
    pub fn level(self, expected: tracing::Level) -> Self {
        assert_eq!(
            self.event.level, expected,
            "level mismatch (target={})",
            self.event.target
        );
        self
    }
}
