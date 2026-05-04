//! Plan v7 Phase 0 / 1.0 — `CellValue` 비교 마이크로벤치.
//!
//! - **Baseline A** (`baseline_a_partialeq_direct`): NaN-unsafe `derive(PartialEq)`
//!   직접 호출. *reference only* — Phase 1.0 에서 prod 빌드는 더 이상 이 경로를
//!   허용하지 않는다 (P3' 게이트).
//! - **Baseline B** (`baseline_b_nan_safe_naive`): Float 만 `total_cmp`, 나머지는
//!   `==`. ceiling 임계값 (<3x / 3-10x / ≥10x) 적용 대상.
//! - **CmpCell** (`cmp_view_canonical_eq`): `canonical_string()` 으로 `Cow<str>`
//!   비교. Plan v7 의 채택안 (Option A).
//!
//! `ferrumgrid` 가 단일 binary crate 라 `use ferrumgrid::types::CellValue` 가
//! 불가능하므로, prod 정의를 mirror 한 `MockCellValue` 와 동일 시그니처의
//! `canonical_string()` 을 본 파일에 둔다. Phase 1.x 에서 lib 분할이 일어나면
//! 진짜 `CellValue` 로 교체 가능.

use std::borrow::Cow;
use std::cmp::Ordering;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Json 변종은 prod `CellValue::Json(serde_json::Value)` 와 타입을 정확히
/// 일치시킨다 (Architect Phase 1.0 Note: serde_json::to_string alloc 비용을
/// 정직하게 측정하기 위해).
#[derive(Debug, Clone, PartialEq)]
enum MockCellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Json(serde_json::Value),
    Timestamp(String),
    Uuid(String),
    Bytes(Vec<u8>),
    Unknown(String),
}

impl MockCellValue {
    fn canonical_string(&self) -> Cow<'_, str> {
        match self {
            MockCellValue::Null => Cow::Borrowed("\0NULL"),
            MockCellValue::Bool(true) => Cow::Borrowed("t"),
            MockCellValue::Bool(false) => Cow::Borrowed("f"),
            MockCellValue::Int(v) => Cow::Owned(v.to_string()),
            MockCellValue::Float(v) => {
                if v.is_nan() {
                    Cow::Borrowed("\0NaN")
                } else if *v == 0.0 {
                    Cow::Borrowed("0")
                } else {
                    Cow::Owned(format!("{v:?}"))
                }
            }
            MockCellValue::Text(s)
            | MockCellValue::Timestamp(s)
            | MockCellValue::Uuid(s)
            | MockCellValue::Unknown(s) => Cow::Borrowed(s.as_str()),
            MockCellValue::Json(v) => {
                Cow::Owned(serde_json::to_string(v).unwrap_or_else(|_| v.to_string()))
            }
            MockCellValue::Bytes(b) => Cow::Owned(format!(
                "\\x{}",
                b.iter().map(|x| format!("{x:02x}")).collect::<String>()
            )),
        }
    }
}

/// Float 만 `total_cmp` 로 NaN-safe, 그 외 variant 는 `derive(PartialEq)` 사용.
fn nan_safe_naive_eq(a: &MockCellValue, b: &MockCellValue) -> bool {
    match (a, b) {
        (MockCellValue::Float(x), MockCellValue::Float(y)) => x.total_cmp(y) == Ordering::Equal,
        _ => a == b,
    }
}

fn sample_pairs() -> Vec<(MockCellValue, MockCellValue)> {
    vec![
        (MockCellValue::Null, MockCellValue::Null),
        (MockCellValue::Bool(true), MockCellValue::Bool(true)),
        (MockCellValue::Int(42), MockCellValue::Int(42)),
        (MockCellValue::Float(3.14), MockCellValue::Float(3.14)),
        (
            MockCellValue::Text("hello".to_string()),
            MockCellValue::Text("hello".to_string()),
        ),
        (
            MockCellValue::Json(serde_json::json!({"k": 1})),
            MockCellValue::Json(serde_json::json!({"k": 1})),
        ),
        (
            MockCellValue::Timestamp("2026-05-04T00:00:00Z".to_string()),
            MockCellValue::Timestamp("2026-05-04T00:00:00Z".to_string()),
        ),
        (
            MockCellValue::Uuid("00000000-0000-0000-0000-000000000001".to_string()),
            MockCellValue::Uuid("00000000-0000-0000-0000-000000000001".to_string()),
        ),
        (
            MockCellValue::Bytes(vec![0xDE, 0xAD]),
            MockCellValue::Bytes(vec![0xDE, 0xAD]),
        ),
        (
            MockCellValue::Unknown("x".to_string()),
            MockCellValue::Unknown("y".to_string()),
        ),
        (MockCellValue::Int(1), MockCellValue::Int(2)),
        (
            MockCellValue::Text("a".to_string()),
            MockCellValue::Text("b".to_string()),
        ),
    ]
}

fn baseline_a_partialeq_direct(c: &mut Criterion) {
    let pairs = sample_pairs();
    c.bench_function("baseline_a_partialeq_direct", |b| {
        b.iter(|| {
            let mut acc = 0_usize;
            for (lhs, rhs) in &pairs {
                if black_box(lhs) == black_box(rhs) {
                    acc += 1;
                }
            }
            black_box(acc)
        });
    });
}

fn baseline_b_nan_safe_naive(c: &mut Criterion) {
    let pairs = sample_pairs();
    c.bench_function("baseline_b_nan_safe_naive", |b| {
        b.iter(|| {
            let mut acc = 0_usize;
            for (lhs, rhs) in &pairs {
                if nan_safe_naive_eq(black_box(lhs), black_box(rhs)) {
                    acc += 1;
                }
            }
            black_box(acc)
        });
    });
}

fn cmp_view_canonical_eq(c: &mut Criterion) {
    let pairs = sample_pairs();
    c.bench_function("cmp_view_canonical_eq", |b| {
        b.iter(|| {
            let mut acc = 0_usize;
            for (lhs, rhs) in &pairs {
                if black_box(lhs).canonical_string() == black_box(rhs).canonical_string() {
                    acc += 1;
                }
            }
            black_box(acc)
        });
    });
}

criterion_group!(
    benches,
    baseline_a_partialeq_direct,
    baseline_b_nan_safe_naive,
    cmp_view_canonical_eq
);
criterion_main!(benches);
