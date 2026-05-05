//! pg_depend recursive 종속성 미리보기 (Plan v7 Phase 2d).
//!
//! Drop CASCADE 실행 전에 사용자가 영향 범위를 인지하도록 종속 객체 LIMIT 51
//! 미리 조회. dependents > 50 시 cutoff label 표시 후 명시 confirm 거치지 않으면
//! 실행 안 함. 미래 Drop UI (table/view/function/role) 의 진입점.

/// pg_depend recursive 미리보기 시 fetch 상한.
///
/// `+1` 은 cutoff 감지용 — 실제 사용자 표시는 `MAX_DISPLAY` 까지.
pub const PREVIEW_FETCH_LIMIT: usize = 51;

/// 사용자에게 표시 가능한 종속 객체 최대 개수. `PREVIEW_FETCH_LIMIT - 1`.
pub const MAX_DISPLAY: usize = 50;

/// 단일 종속 객체 (pg_depend 1행).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependent {
    /// 종속 객체 OID (pg_depend.objid).
    pub objid: u32,
    /// 종속 객체의 schema-qualified 표시 이름 (예: "public.orders_view").
    pub display_name: String,
    /// 객체 종류 (예: "table", "view", "function", "role").
    pub object_kind: String,
    /// pg_depend.deptype (n=normal, a=auto, i=internal, e=extension, ...).
    pub deptype: char,
}

/// `Drop` 미리보기 결과 — 종속 객체 리스트 + cutoff 플래그.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DependentList {
    /// 표시할 종속 객체 (최대 `MAX_DISPLAY` 개).
    pub items: Vec<Dependent>,
    /// `true` 이면 결과가 `MAX_DISPLAY` 를 초과 (실제 종속 객체가 더 있음).
    pub truncated: bool,
}

#[allow(dead_code)]
impl DependentList {
    /// `Vec<Dependent>` 를 받아 `MAX_DISPLAY` 로 절단 + truncated 플래그.
    /// 입력이 `PREVIEW_FETCH_LIMIT` (51) 이상이면 truncated=true.
    pub fn from_fetched(mut fetched: Vec<Dependent>) -> Self {
        let truncated = fetched.len() >= PREVIEW_FETCH_LIMIT;
        fetched.truncate(MAX_DISPLAY);
        Self {
            items: fetched,
            truncated,
        }
    }

    /// 종속 객체가 없으면 `true` — Drop CASCADE 가 사실상 단일 객체 drop 과 동등.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 사용자에게 표시할 cutoff label.
    /// truncated=true → "Showing first 50 of more than 50 dependents (truncated)".
    /// 그 외 → "Showing all N dependents".
    pub fn cutoff_label(&self) -> String {
        if self.truncated {
            format!(
                "Showing first {} of more than {} dependents (truncated)",
                MAX_DISPLAY, MAX_DISPLAY
            )
        } else {
            format!("Showing all {} dependents", self.items.len())
        }
    }
}

/// pg_depend recursive CTE — refobjid 가 가리키는 객체에 대한 *모든* 종속 객체
/// (transitive) 를 LIMIT 51 까지 조회. `$1` = root object OID (pg_class.oid 등).
///
/// deptype filter:
/// - 'n' (normal) — DROP 시 자동 cascade
/// - 'a' (auto) — DROP 시 자동 함께 삭제
/// - 'i' (internal) — 일반적으로 무시 (시스템)
/// - 'e' (extension) — 확장 의존성
///
/// 본 query 는 'n' / 'a' 만 노출 (사용자 영향).
#[allow(dead_code)]
pub fn pg_depend_recursive_sql() -> &'static str {
    "WITH RECURSIVE deps AS ( \
       SELECT classid, objid, objsubid, refclassid, refobjid, refobjsubid, deptype, 1 AS depth \
         FROM pg_catalog.pg_depend \
        WHERE refobjid = $1 AND deptype IN ('n', 'a') \
       UNION \
       SELECT d.classid, d.objid, d.objsubid, d.refclassid, d.refobjid, d.refobjsubid, d.deptype, deps.depth + 1 \
         FROM pg_catalog.pg_depend d \
         JOIN deps ON d.refobjid = deps.objid \
        WHERE d.deptype IN ('n', 'a') AND deps.depth < 8 \
     ) \
     SELECT objid, classid, objsubid, deptype FROM deps LIMIT 51"
}

/// classid (pg_depend) → object kind 문자열 매핑.
/// 가장 흔한 catalog OID 만 cover. 알 수 없으면 "object".
#[allow(dead_code)]
pub fn classify_object(classid: u32) -> &'static str {
    match classid {
        1259 => "table_or_view", // pg_class
        1255 => "function",      // pg_proc
        1247 => "type",          // pg_type
        1417 => "foreign_server",
        2615 => "schema",          // pg_namespace
        2617 => "operator",        // pg_operator
        3079 => "extension",       // pg_extension
        1260 => "role",            // pg_authid
        2606 => "constraint",      // pg_constraint
        1249 => "table_attribute", // pg_attribute
        _ => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dep(idx: u32) -> Dependent {
        Dependent {
            objid: 10000 + idx,
            display_name: format!("public.dep_{idx}"),
            object_kind: "table".to_string(),
            deptype: 'n',
        }
    }

    #[test]
    fn empty_list_reports_empty_and_zero_count() {
        let list = DependentList::from_fetched(vec![]);
        assert!(list.is_empty());
        assert!(!list.truncated);
        assert_eq!(list.items.len(), 0);
        assert_eq!(list.cutoff_label(), "Showing all 0 dependents");
    }

    #[test]
    fn one_dep_below_cutoff_not_truncated() {
        let list = DependentList::from_fetched(vec![make_dep(1)]);
        assert_eq!(list.items.len(), 1);
        assert!(!list.truncated);
        assert_eq!(list.cutoff_label(), "Showing all 1 dependents");
    }

    #[test]
    fn fifty_deps_at_max_not_truncated() {
        let deps: Vec<_> = (0..50).map(make_dep).collect();
        let list = DependentList::from_fetched(deps);
        assert_eq!(list.items.len(), 50);
        assert!(!list.truncated);
        assert_eq!(list.cutoff_label(), "Showing all 50 dependents");
    }

    #[test]
    fn fifty_one_deps_triggers_truncated_flag() {
        let deps: Vec<_> = (0..51).map(make_dep).collect();
        let list = DependentList::from_fetched(deps);
        assert_eq!(list.items.len(), 50);
        assert!(list.truncated);
        assert_eq!(
            list.cutoff_label(),
            "Showing first 50 of more than 50 dependents (truncated)"
        );
    }

    #[test]
    fn over_fifty_one_deps_still_capped_at_fifty_display() {
        let deps: Vec<_> = (0..200).map(make_dep).collect();
        let list = DependentList::from_fetched(deps);
        assert_eq!(list.items.len(), MAX_DISPLAY);
        assert!(list.truncated);
    }

    #[test]
    fn pg_depend_sql_includes_recursive_cte_and_limit_51() {
        let sql = pg_depend_recursive_sql();
        assert!(sql.contains("WITH RECURSIVE"));
        assert!(sql.contains("pg_catalog.pg_depend"));
        assert!(sql.contains("refobjid = $1"));
        assert!(sql.contains("LIMIT 51"));
        // deptype filter
        assert!(sql.contains("deptype IN ('n', 'a')"));
        // recursion guard
        assert!(sql.contains("depth < 8"));
    }

    #[test]
    fn pg_depend_sql_is_single_select() {
        let sql = pg_depend_recursive_sql();
        // 단일 SELECT — semicolon 없음 (parameterized prepared statement 용)
        assert!(!sql.contains(';'));
    }

    #[test]
    fn classify_object_known_classids() {
        assert_eq!(classify_object(1259), "table_or_view");
        assert_eq!(classify_object(1255), "function");
        assert_eq!(classify_object(1260), "role");
        assert_eq!(classify_object(2615), "schema");
    }

    #[test]
    fn classify_object_unknown_returns_default() {
        assert_eq!(classify_object(99999), "object");
    }

    #[test]
    fn preview_fetch_limit_is_one_more_than_max_display() {
        assert_eq!(PREVIEW_FETCH_LIMIT, MAX_DISPLAY + 1);
    }

    #[test]
    fn dep_eq_compares_all_fields() {
        let a = make_dep(1);
        let b = make_dep(1);
        let c = make_dep(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
