use std::{borrow::Cow, fmt, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConnectionId(pub uuid::Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionConfig {
    pub id: ConnectionId,
    pub display_name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(default)]
    pub password: String,
    pub use_tls: bool,
    pub color_tag: Option<String>,
    /// 선택적 폴더/그룹명 (dev/staging/prod 등). None/빈 문자열 = 미분류.
    #[serde(default)]
    pub group: Option<String>,
    pub ssh_tunnel: Option<SshTunnelConfig>,
}

/// pg_dump 출력 형식. Plan v7 Phase 4a — `Tar` 추가.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BackupFormat {
    Custom,
    Plain,
    /// `pg_dump --format=tar` — 단일 tar archive (Phase 4a 신설).
    /// `pg_restore` 가능하지만 Custom 보다 압축률 떨어짐.
    Tar,
    /// FerrumGrid built-in SQL backup engine — does NOT shell out to `pg_dump`.
    /// Streams DDL + COPY data directly via tokio-postgres.
    SqlOnly,
    /// FerrumGrid proprietary binary backup engine (.fgb).
    Fgb,
}

impl BackupFormat {
    #[allow(dead_code)]
    pub const BACKUP_TAB_OPTIONS: [Self; 5] = [Self::Fgb, Self::SqlOnly, Self::Custom, Self::Plain, Self::Tar];

    pub fn label(self) -> &'static str {
        match self {
            Self::Custom => "Custom archive",
            Self::Plain => "Plain SQL",
            Self::Tar => "Tar archive",
            Self::SqlOnly => "SQL (built-in)",
            Self::Fgb => "FerrumGrid Backup (.fgb)",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Custom => "dump",
            Self::Plain => "sql",
            Self::Tar => "tar",
            Self::SqlOnly => "sql",
            Self::Fgb => "fgb",
        }
    }

    /// pg_dump `--format` flag value.
    ///
    /// **Caller contract**: never invoked for `SqlOnly` or `Fgb` — the dispatcher in
    /// `crate::db::backup::run_backup` routes them to their respective engines
    /// before this method is reached. Returns `""` defensively.
    pub fn pg_dump_format(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Plain => "plain",
            Self::Tar => "tar",
            Self::SqlOnly => "",
            Self::Fgb => "",
        }
    }
}

/// Backup 진행 상태 (Plan v7 Phase 4a). DiagnosticsPanel 에 노출.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BackupStatus {
    /// 진행 중 backup 없음.
    Idle,
    /// 진행 중. `pct` 는 0..=100 (pg_dump 가 progress 를 stderr 로 보고하지 않으므로
    /// 현재는 simple inferred — Phase 4b 에서 정확한 progress reporting 추가).
    InProgress { pct: u8 },
    /// 정상 완료.
    Done,
    /// 실패.
    Failed { error: String },
}

#[allow(dead_code)]
impl BackupStatus {
    /// 진행 중인지 (UI spinner 표시 용도).
    pub fn is_active(&self) -> bool {
        matches!(self, BackupStatus::InProgress { .. })
    }

    /// 사용자에게 표시할 짧은 라벨.
    pub fn short_label(&self) -> &'static str {
        match self {
            BackupStatus::Idle => "Idle",
            BackupStatus::InProgress { .. } => "Running",
            BackupStatus::Done => "Done",
            BackupStatus::Failed { .. } => "Failed",
        }
    }
}

/// Plan v7 Phase 4a — Backup 의 *통합 정보 schema* (DiagnosticsPanel + 후속 phase 의
/// CLI integration 진입점). 기존 `BackupRecord` (완료된 것만) 와 달리 진행 중 status
/// 까지 포함.
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupInfoV1 {
    pub format: BackupFormat,
    /// 출력 인코딩 (예: "UTF-8").
    pub encoding: String,
    pub status: BackupStatus,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub bytes_total: Option<u64>,
    pub bytes_done: u64,
    pub eta_seconds: Option<u64>,
}

impl Default for BackupInfoV1 {
    fn default() -> Self {
        Self {
            format: BackupFormat::Custom,
            encoding: "UTF-8".to_string(),
            status: BackupStatus::Idle,
            started_at: None,
            finished_at: None,
            bytes_total: None,
            bytes_done: 0,
            eta_seconds: None,
        }
    }
}

/// Plan v7 Phase 4a / US-P4a2 — `AppState` 의 backup 관련 필드들을 통합한
/// `BackupInfoV1` 을 derive.
///
/// 우선순위:
/// 1. `running` → `InProgress { pct: 0 }` (정확한 pct 는 Phase 4b 에서 pg_dump
///    progress 통합 후 갱신)
/// 2. `last_error` → `Failed { error }`
/// 3. `last_record` → `Done` (직전 backup 의 size/timestamp 노출)
/// 4. 그 외 → `Idle`
#[allow(dead_code)]
pub fn current_backup_info(
    format: BackupFormat,
    running: bool,
    last_error: Option<&str>,
    last_record: Option<&BackupRecord>,
) -> BackupInfoV1 {
    let status = if running {
        BackupStatus::InProgress { pct: 0 }
    } else if let Some(err) = last_error {
        BackupStatus::Failed {
            error: err.to_string(),
        }
    } else if last_record.is_some() {
        BackupStatus::Done
    } else {
        BackupStatus::Idle
    };

    let (bytes_done, finished_at, total_format) = match (running, last_record) {
        (false, Some(record)) => (
            record.size_bytes,
            Some(record.completed_at.clone()),
            record.format,
        ),
        _ => (0, None, format),
    };

    BackupInfoV1 {
        format: total_format,
        encoding: "UTF-8".to_string(),
        status,
        started_at: None, // Phase 4b: 실제 backup 시작 시각 추적 시 채움
        finished_at,
        bytes_total: None, // pg_dump 가 progress 보고 안 함
        bytes_done,
        eta_seconds: None, // Phase 4b: bytes_done / elapsed 로 추정 시 채움
    }
}

#[derive(Debug, Clone)]
pub struct BackupRequest {
    pub conn_id: ConnectionId,
    pub config: ConnectionConfig,
    pub output_dir: PathBuf,
    pub schema: Option<String>,
    pub format: BackupFormat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupRecord {
    pub conn_id: ConnectionId,
    pub connection_name: String,
    pub database: String,
    pub schema: Option<String>,
    pub format: BackupFormat,
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub duration_ms: u128,
    pub completed_at: String,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            id: ConnectionId::new(),
            display_name: String::new(),
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            use_tls: false,
            color_tag: None,
            group: None,
            ssh_tunnel: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<CellValue>>,
    pub execution_time_ms: u128,
}

#[derive(Debug, Clone)]
pub struct ColumnMeta {
    pub name: String,
    pub type_name: String,
}

/// 셀 값.
///
/// **`PartialEq` 정책 (Plan v7 P3' Wire-canonical comparison)**: 비교는 항상
/// [`CellValue::cmp_view`] 가 반환하는 [`CmpCell`] 을 경유한다. 직접 `==` 호출
/// 은 NaN-unsafe (Float NaN != NaN) 이므로 prod 빌드에서 차단되며,
/// `#[cfg(test)]` 환경에서만 wire-canonical 정의를 따르는 [`PartialEq`] 가
/// 노출된다 (단위 테스트 fixture 비교용).
#[derive(Debug, Clone)]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Json(serde_json::Value),
    Timestamp(String),
    Uuid(uuid::Uuid),
    Bytes(Vec<u8>),
    Unknown(String),
}

/// Wire-canonical 비교/해시용 newtype.
///
/// Plan v7 P3' / ADR-1 — `PartialEq` / `Eq` / `Hash` 가 정규화된 문자열에
/// 대해 정의되므로 NaN-safe 하고 (NaN/NaN 동등), `RowKey` 의 hash 입력으로
/// 재사용 가능하다.
///
/// Phase 1.0 단계에서는 prod 호출 site 가 아직 없으므로 `#[allow(dead_code)]`
/// 로 의도적 미사용을 명시한다. Phase 1.1 에서 `apply_data_edits` 시그니처
/// 변경 + `EditableCell::is_dirty` 가 이 경로로 전환되면 attribute 는 자연스레
/// 제거된다.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmpCell<'a> {
    pub canonical: Cow<'a, str>,
}

#[allow(dead_code)]
impl CellValue {
    /// 정규화된 wire-form 문자열을 반환한다.
    ///
    /// Plan v7 §7 In-memory ↔ Wire 정규화 표 기반.
    /// - `Null` → `"\0NULL"` (sentinel; 빈 문자열 ≠ Null 구분)
    /// - `Bool(b)` → `"t"` / `"f"`
    /// - `Int(i)` → `i.to_string()`
    /// - `Float(NaN)` → `"\0NaN"` (고정)
    /// - `Float(±0.0)` → `"0"` (sign drop)
    /// - `Float(f)` → Rust shortest round-trip (`{f:?}`)
    /// - `Text(s)` / `Timestamp(s)` / `Unknown(s)` → 그대로 빌림
    /// - `Json(v)` → `serde_json::to_string(&v)` (서버 응답 포맷 그대로 — 키 정렬은 Phase 1.1+ 보강)
    /// - `Uuid(u)` → `u.to_string()`
    /// - `Bytes(b)` → `\x<lowercase hex>`
    pub fn canonical_string(&self) -> Cow<'_, str> {
        match self {
            CellValue::Null => Cow::Borrowed("\0NULL"),
            CellValue::Bool(true) => Cow::Borrowed("t"),
            CellValue::Bool(false) => Cow::Borrowed("f"),
            CellValue::Int(v) => Cow::Owned(v.to_string()),
            CellValue::Float(v) => {
                if v.is_nan() {
                    Cow::Borrowed("\0NaN")
                } else if *v == 0.0 {
                    Cow::Borrowed("0")
                } else {
                    Cow::Owned(format!("{v:?}"))
                }
            }
            CellValue::Text(s) | CellValue::Timestamp(s) | CellValue::Unknown(s) => {
                Cow::Borrowed(s.as_str())
            }
            CellValue::Json(v) => Cow::Owned(json_canonical(v)),
            CellValue::Uuid(u) => Cow::Owned(u.to_string()),
            CellValue::Bytes(b) => Cow::Owned(format!("\\x{}", hex_encode(b))),
        }
    }

    /// 비교/해시 가능한 view 를 반환한다 (Plan v7 P3' 게이트).
    pub fn cmp_view(&self) -> CmpCell<'_> {
        CmpCell {
            canonical: self.canonical_string(),
        }
    }
}

/// 테스트 전용 `PartialEq` — wire-canonical 의미를 따른다.
///
/// Plan v7 ADR-1: prod 빌드에서는 `derive(PartialEq)` 가 부재하므로 `==` 직접
/// 호출이 컴파일 에러로 차단된다. 단위 테스트는 fixture 비교에 자주 의존하므로
/// `#[cfg(test)]` 에서만 동등성을 노출한다.
#[cfg(test)]
impl PartialEq for CellValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_view() == other.cmp_view()
    }
}

#[cfg(test)]
mod cell_value_canonical_tests {
    use super::*;

    #[test]
    fn null_canonical_uses_sentinel_not_empty() {
        assert_eq!(CellValue::Null.canonical_string(), "\0NULL");
        assert_ne!(CellValue::Null.canonical_string(), "");
        assert_ne!(
            CellValue::Null.canonical_string(),
            CellValue::Text(String::new()).canonical_string()
        );
    }

    #[test]
    fn float_nan_is_reflexive_via_cmp_view() {
        let a = CellValue::Float(f64::NAN);
        let b = CellValue::Float(f64::NAN);
        assert_eq!(a.cmp_view(), b.cmp_view(), "NaN must equal NaN via CmpCell");
    }

    #[test]
    fn float_positive_and_negative_zero_collapse() {
        let pos = CellValue::Float(0.0);
        let neg = CellValue::Float(-0.0);
        assert_eq!(pos.cmp_view(), neg.cmp_view(), "+0.0 and -0.0 are wire-equal");
    }

    #[test]
    fn float_inf_distinct_from_neg_inf() {
        let inf = CellValue::Float(f64::INFINITY);
        let ninf = CellValue::Float(f64::NEG_INFINITY);
        assert_ne!(inf.cmp_view(), ninf.cmp_view());
    }

    #[test]
    fn bool_canonical_is_pg_text_form() {
        assert_eq!(CellValue::Bool(true).canonical_string(), "t");
        assert_eq!(CellValue::Bool(false).canonical_string(), "f");
    }

    #[test]
    fn bytes_canonical_uses_lowercase_hex_with_prefix() {
        let cell = CellValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(cell.canonical_string(), "\\xdeadbeef");
    }

    #[test]
    fn text_borrows_without_alloc() {
        let cell = CellValue::Text("hello".to_string());
        let canonical = cell.canonical_string();
        assert!(matches!(canonical, Cow::Borrowed(_)));
        assert_eq!(canonical, "hello");
    }

    #[test]
    fn cell_value_partial_eq_wraps_canonical_in_cfg_test() {
        // cfg(test) PartialEq impl 가 wire-canonical 의미를 따르는지 검증.
        assert_eq!(CellValue::Float(f64::NAN), CellValue::Float(f64::NAN));
        assert_eq!(CellValue::Float(0.0), CellValue::Float(-0.0));
        assert_ne!(CellValue::Int(1), CellValue::Int(2));
    }

    /// Plan v7 P3' compile-fail constraint 의 *서면 + mechanical* evidence.
    ///
    /// 단일 binary crate (lib.rs 부재) 환경에서는 `trybuild` / doc test 로
    /// 컴파일 실패를 별도 파일로 검증할 수 없다. 대신:
    ///
    /// 1. **Mechanical 1차 게이트**: prod 빌드 (`cargo build --release`) 가
    ///    `CellValue: !PartialEq` 이므로 직접 `==` 사용 site 가 있으면 실패.
    ///    Architect Phase 1.0 review 에서 직접 실행하여 검증 (exit 0).
    /// 2. **CI 2차 게이트** (Makefile / .github/workflows 에 추가 권장):
    ///    ```sh
    ///    ! grep -RnE '\b(cell|value)[a-z_]*\s*==\s*&?(cell|value|CellValue)' src/
    ///    ```
    /// 3. **본 테스트**: 게이트 정책의 *서면 evidence* — 본 docstring 이
    ///    Phase 1.1+ reviewer 가 정책 컨텍스트를 즉시 회복할 진입점.
    #[test]
    fn compile_fail_constraint_is_documented() {
        // 의도적 no-op — 본 함수의 doc comment 가 evidence 역할.
    }

    #[test]
    fn json_canonical_is_key_order_independent() {
        let unsorted: serde_json::Value =
            serde_json::from_str(r#"{"b":1,"a":2,"c":{"y":2,"x":1}}"#).unwrap();
        let sorted: serde_json::Value =
            serde_json::from_str(r#"{"a":2,"b":1,"c":{"x":1,"y":2}}"#).unwrap();

        let lhs = CellValue::Json(unsorted);
        let rhs = CellValue::Json(sorted);

        assert_eq!(
            lhs.cmp_view(),
            rhs.cmp_view(),
            "JSON canonical 은 키 정렬 무관"
        );
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Null => write!(f, "NULL"),
            CellValue::Bool(v) => write!(f, "{v}"),
            CellValue::Int(v) => write!(f, "{v}"),
            CellValue::Float(v) => write!(f, "{v}"),
            CellValue::Text(v) => write!(f, "{v}"),
            CellValue::Json(v) => write!(f, "{v}"),
            CellValue::Timestamp(v) => write!(f, "{v}"),
            CellValue::Uuid(v) => write!(f, "{v}"),
            CellValue::Bytes(v) => write!(f, "\\x{}", hex_encode(v)),
            CellValue::Unknown(v) => write!(f, "{v}"),
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `serde_json::Value` 의 키 정렬 정규화 직렬화.
///
/// Plan v7 §7 — Json canonical 은 키 정렬 순서로 minified. `RowKey` (blake3)
/// 의 안정성 prerequisite. Architect Phase 1.0 review 의 Note (Phase 1.1 이전
/// 보강 필수) 즉시 반영.
fn json_canonical(value: &serde_json::Value) -> String {
    use std::collections::BTreeMap;

    fn walk(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Object(map) => {
                // BTreeMap<String, Value> 로 한 번에 정렬 + 키 owned, 곧장 Object 로 collect.
                let sorted: BTreeMap<String, serde_json::Value> =
                    map.iter().map(|(k, v)| (k.clone(), walk(v))).collect();
                serde_json::Value::Object(sorted.into_iter().collect())
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(walk).collect())
            }
            other => other.clone(),
        }
    }

    serde_json::to_string(&walk(value)).unwrap_or_else(|_| value.to_string())
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub name: String,
    pub source_schema: String,
    pub source_table: String,
    pub source_column: String,
    pub target_schema: String,
    pub target_table: String,
    pub target_column: String,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub table_type: String,
    /// US-K2 — pg_class.oid (Plan v7 cache invalidation 매칭 키). None 이면 fetch
    /// 실패 또는 미지원 catalog (e.g., information_schema view).
    pub oid: Option<u32>,
    pub row_estimate: Option<u64>,
    /// COMMENT ON TABLE/VIEW (obj_description). None 이면 설명 없음.
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub enum_values: Vec<String>,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    /// COMMENT ON COLUMN (col_description). None 이면 설명 없음.
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String,
}

#[derive(Debug, Clone)]
pub struct RuleInfo {
    pub name: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct TriggerInfo {
    pub name: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub schema: String,
    pub name: String,
    pub arguments: String,
    pub return_type: String,
    pub kind: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct RoleInfo {
    pub name: String,
    pub can_login: bool,
    pub is_superuser: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub can_replicate: bool,
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub id: uuid::Uuid,
    pub title: String,
    pub content: String,
    pub connection_id: Option<ConnectionId>,
}

impl EditorTab {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: title.into(),
            content: String::new(),
            connection_id: None,
        }
    }
}

#[cfg(test)]
mod backup_info_tests {
    use super::*;

    #[test]
    fn backup_format_tar_label_and_extension() {
        assert_eq!(BackupFormat::Tar.label(), "Tar archive");
        assert_eq!(BackupFormat::Tar.extension(), "tar");
        assert_eq!(BackupFormat::Tar.pg_dump_format(), "tar");
    }

    #[test]
    fn backup_format_custom_unchanged() {
        assert_eq!(BackupFormat::Custom.label(), "Custom archive");
        assert_eq!(BackupFormat::Custom.extension(), "dump");
        assert_eq!(BackupFormat::Custom.pg_dump_format(), "custom");
    }

    #[test]
    fn backup_format_plain_unchanged() {
        assert_eq!(BackupFormat::Plain.label(), "Plain SQL");
        assert_eq!(BackupFormat::Plain.extension(), "sql");
        assert_eq!(BackupFormat::Plain.pg_dump_format(), "plain");
    }

    #[test]
    fn backup_format_sql_only_label_and_extension() {
        assert_eq!(BackupFormat::SqlOnly.label(), "SQL (built-in)");
        assert_eq!(BackupFormat::SqlOnly.extension(), "sql");
    }

    #[test]
    fn backup_format_sql_only_pg_dump_format_is_empty_sentinel() {
        // Contract: dispatcher in db::backup::run_backup must route SqlOnly
        // before pg_dump_format() is reached. The empty-string return value is
        // a defensive sentinel — feeding it to pg_dump would fail loudly anyway.
        assert_eq!(BackupFormat::SqlOnly.pg_dump_format(), "");
    }

    #[test]
    fn backup_format_sql_only_distinct_from_plain() {
        assert_ne!(BackupFormat::SqlOnly, BackupFormat::Plain);
        assert_eq!(BackupFormat::SqlOnly.extension(), BackupFormat::Plain.extension());
        assert_ne!(BackupFormat::SqlOnly.label(), BackupFormat::Plain.label());
    }

    #[test]
    fn backup_tab_options_include_built_in_engine() {
        assert_eq!(BackupFormat::BACKUP_TAB_OPTIONS[0], BackupFormat::Fgb);
        assert!(BackupFormat::BACKUP_TAB_OPTIONS.contains(&BackupFormat::SqlOnly));
        assert!(BackupFormat::BACKUP_TAB_OPTIONS.contains(&BackupFormat::Custom));
        assert!(BackupFormat::BACKUP_TAB_OPTIONS.contains(&BackupFormat::Plain));
        assert!(BackupFormat::BACKUP_TAB_OPTIONS.contains(&BackupFormat::Tar));
    }

    #[test]
    fn backup_status_in_progress_is_active() {
        assert!(BackupStatus::InProgress { pct: 0 }.is_active());
        assert!(BackupStatus::InProgress { pct: 50 }.is_active());
        assert!(BackupStatus::InProgress { pct: 100 }.is_active());
    }

    #[test]
    fn backup_status_idle_done_failed_not_active() {
        assert!(!BackupStatus::Idle.is_active());
        assert!(!BackupStatus::Done.is_active());
        assert!(!BackupStatus::Failed {
            error: "oops".to_string()
        }
        .is_active());
    }

    #[test]
    fn backup_status_short_labels_distinct() {
        assert_eq!(BackupStatus::Idle.short_label(), "Idle");
        assert_eq!(
            BackupStatus::InProgress { pct: 42 }.short_label(),
            "Running"
        );
        assert_eq!(BackupStatus::Done.short_label(), "Done");
        assert_eq!(
            BackupStatus::Failed {
                error: "x".to_string()
            }
            .short_label(),
            "Failed"
        );
    }

    #[test]
    fn backup_info_v1_default_is_idle_zero_bytes() {
        let info = BackupInfoV1::default();
        assert_eq!(info.status, BackupStatus::Idle);
        assert_eq!(info.bytes_done, 0);
        assert!(info.bytes_total.is_none());
        assert!(info.eta_seconds.is_none());
        assert!(info.started_at.is_none());
        assert!(info.finished_at.is_none());
        assert_eq!(info.encoding, "UTF-8");
        assert_eq!(info.format, BackupFormat::Custom);
    }

    #[test]
    fn backup_info_v1_serde_round_trip_idle() {
        let original = BackupInfoV1::default();
        let serialized = serde_json::to_string(&original).expect("serialize");
        let restored: BackupInfoV1 = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(restored.status, BackupStatus::Idle);
        assert_eq!(restored.format, BackupFormat::Custom);
        assert_eq!(restored.encoding, "UTF-8");
    }

    #[test]
    fn backup_info_v1_serde_round_trip_in_progress() {
        let original = BackupInfoV1 {
            format: BackupFormat::Tar,
            encoding: "UTF-8".to_string(),
            status: BackupStatus::InProgress { pct: 75 },
            started_at: Some("2026-05-05T01:30:00Z".to_string()),
            finished_at: None,
            bytes_total: Some(1_048_576),
            bytes_done: 786_432,
            eta_seconds: Some(8),
        };
        let serialized = serde_json::to_string(&original).expect("serialize");
        let restored: BackupInfoV1 = serde_json::from_str(&serialized).expect("deserialize");
        assert!(matches!(
            restored.status,
            BackupStatus::InProgress { pct: 75 }
        ));
        assert_eq!(restored.format, BackupFormat::Tar);
        assert_eq!(restored.bytes_done, 786_432);
        assert_eq!(restored.eta_seconds, Some(8));
    }

    fn make_record(size: u64, completed: &str) -> BackupRecord {
        BackupRecord {
            conn_id: ConnectionId::new(),
            connection_name: "test".to_string(),
            database: "db".to_string(),
            schema: None,
            format: BackupFormat::Custom,
            file_path: PathBuf::from("/tmp/x.dump"),
            size_bytes: size,
            duration_ms: 100,
            completed_at: completed.to_string(),
        }
    }

    #[test]
    fn current_backup_info_idle_when_no_state() {
        let info = current_backup_info(BackupFormat::Custom, false, None, None);
        assert_eq!(info.status, BackupStatus::Idle);
        assert_eq!(info.bytes_done, 0);
        assert!(info.finished_at.is_none());
    }

    #[test]
    fn current_backup_info_in_progress_when_running() {
        let info = current_backup_info(BackupFormat::Tar, true, None, None);
        assert_eq!(info.status, BackupStatus::InProgress { pct: 0 });
        assert_eq!(info.format, BackupFormat::Tar);
    }

    #[test]
    fn current_backup_info_failed_takes_priority_over_history() {
        let record = make_record(123, "2026-05-04T00:00:00Z");
        let info = current_backup_info(
            BackupFormat::Plain,
            false,
            Some("disk full"),
            Some(&record),
        );
        if let BackupStatus::Failed { error } = info.status {
            assert_eq!(error, "disk full");
        } else {
            panic!("expected Failed, got {:?}", info.status);
        }
    }

    #[test]
    fn current_backup_info_done_with_history_exposes_size_and_format() {
        let record = make_record(2048, "2026-05-05T01:30:00Z");
        let info = current_backup_info(BackupFormat::Plain, false, None, Some(&record));
        assert_eq!(info.status, BackupStatus::Done);
        assert_eq!(info.bytes_done, 2048);
        assert_eq!(info.finished_at.as_deref(), Some("2026-05-05T01:30:00Z"));
        assert_eq!(info.format, BackupFormat::Custom); // record.format 우선
    }

    #[test]
    fn current_backup_info_running_overrides_history() {
        // running 이 true 면 history 가 있어도 InProgress.
        let record = make_record(1024, "2026-05-04T00:00:00Z");
        let info = current_backup_info(BackupFormat::Tar, true, None, Some(&record));
        assert_eq!(info.status, BackupStatus::InProgress { pct: 0 });
        // bytes_done 0 (Phase 4b 에서 정확한 progress 추적)
        assert_eq!(info.bytes_done, 0);
    }

    #[test]
    fn backup_info_v1_serde_round_trip_failed() {
        let original = BackupInfoV1 {
            format: BackupFormat::Plain,
            encoding: "UTF-8".to_string(),
            status: BackupStatus::Failed {
                error: "permission denied".to_string(),
            },
            started_at: Some("2026-05-05T01:30:00Z".to_string()),
            finished_at: Some("2026-05-05T01:30:05Z".to_string()),
            bytes_total: None,
            bytes_done: 0,
            eta_seconds: None,
        };
        let serialized = serde_json::to_string(&original).expect("serialize");
        let restored: BackupInfoV1 = serde_json::from_str(&serialized).expect("deserialize");
        if let BackupStatus::Failed { error } = restored.status {
            assert_eq!(error, "permission denied");
        } else {
            panic!("expected Failed");
        }
    }
}
