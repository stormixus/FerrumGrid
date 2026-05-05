//! Query 탭의 명시 BEGIN/COMMIT/ROLLBACK SQL 검출.
//!
//! Plan v7 Phase 3b — Query 탭에서 사용자가 직접 `BEGIN` / `START TRANSACTION` 등
//! 을 실행하면 dangling transaction 추적이 활성화되어야 한다. 본 모듈은 SQL
//! 텍스트를 분석하여 transaction 경계를 식별 — `dangling_tx` infra 의 입력원.

/// 사용자가 입력한 SQL 의 명시 transaction 경계 분류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplicitTxClass {
    /// `BEGIN` / `START TRANSACTION` — explicit tx 시작.
    Begin,
    /// `COMMIT` / `END` — explicit tx 종료 (commit).
    Commit,
    /// `ROLLBACK` / `ABORT` — explicit tx 종료 (rollback).
    Rollback,
    /// `SAVEPOINT name` — savepoint 생성 (tx 활성 가정).
    Savepoint,
    /// 위 키워드와 무관한 일반 SQL.
    None,
}

/// SQL 의 leading whitespace 와 SQL line/block comment 를 제거하여
/// 첫 키워드를 대문자로 추출.
fn first_keyword_uppercase(sql: &str) -> Option<String> {
    let stripped = strip_leading_comments_and_whitespace(sql);
    let first = stripped.split_whitespace().next()?;
    Some(first.to_uppercase())
}

/// SQL 시작부의 whitespace + `-- ...` line comment + `/* ... */` block comment 제거.
fn strip_leading_comments_and_whitespace(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if let Some(rest) = s.strip_prefix("--") {
            // line comment: \n 까지 skip
            if let Some(idx) = rest.find('\n') {
                s = rest[idx + 1..].trim_start();
            } else {
                return "";
            }
        } else if let Some(rest) = s.strip_prefix("/*") {
            // block comment: */ 까지 skip
            if let Some(idx) = rest.find("*/") {
                s = rest[idx + 2..].trim_start();
            } else {
                return "";
            }
        } else {
            return s;
        }
    }
}

/// SQL 의 명시 transaction 분류.
///
/// 매칭 규칙 (대소문자 무관, leading whitespace/comment 무시):
/// - `BEGIN` / `BEGIN ;` / `BEGIN TRANSACTION` / `BEGIN WORK` / `START TRANSACTION` → `Begin`
/// - `COMMIT` / `END` / `COMMIT WORK` → `Commit`
/// - `ROLLBACK` / `ABORT` / `ROLLBACK WORK` → `Rollback`
/// - `SAVEPOINT name` → `Savepoint`
/// - 그 외 → `None`
pub fn classify_explicit_tx(sql: &str) -> ExplicitTxClass {
    let Some(first) = first_keyword_uppercase(sql) else {
        return ExplicitTxClass::None;
    };
    // strip trailing semicolon for single-token statements like "BEGIN;"
    let first = first.trim_end_matches(';');
    match first {
        "BEGIN" => ExplicitTxClass::Begin,
        "START" => {
            // require following "TRANSACTION" token
            let stripped = strip_leading_comments_and_whitespace(sql);
            let mut tokens = stripped.split_whitespace();
            tokens.next(); // skip START
            match tokens.next().map(str::to_uppercase).as_deref() {
                Some("TRANSACTION") => ExplicitTxClass::Begin,
                _ => ExplicitTxClass::None,
            }
        }
        "COMMIT" | "END" => ExplicitTxClass::Commit,
        "ROLLBACK" | "ABORT" => ExplicitTxClass::Rollback,
        "SAVEPOINT" => ExplicitTxClass::Savepoint,
        _ => ExplicitTxClass::None,
    }
}

/// `classify_explicit_tx(sql) == Begin` 의 편의 alias.
#[allow(dead_code)]
pub fn is_explicit_begin(sql: &str) -> bool {
    matches!(classify_explicit_tx(sql), ExplicitTxClass::Begin)
}

/// `classify_explicit_tx(sql)` 가 Commit 또는 Rollback 인지.
#[allow(dead_code)]
pub fn is_explicit_tx_end(sql: &str) -> bool {
    matches!(
        classify_explicit_tx(sql),
        ExplicitTxClass::Commit | ExplicitTxClass::Rollback
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_begin_is_classified() {
        assert_eq!(classify_explicit_tx("BEGIN"), ExplicitTxClass::Begin);
        assert_eq!(classify_explicit_tx("begin"), ExplicitTxClass::Begin);
        assert_eq!(classify_explicit_tx("BEGIN;"), ExplicitTxClass::Begin);
    }

    #[test]
    fn begin_transaction_variants() {
        assert_eq!(classify_explicit_tx("BEGIN TRANSACTION"), ExplicitTxClass::Begin);
        assert_eq!(classify_explicit_tx("BEGIN WORK"), ExplicitTxClass::Begin);
        assert_eq!(classify_explicit_tx("START TRANSACTION"), ExplicitTxClass::Begin);
        assert_eq!(classify_explicit_tx("start transaction"), ExplicitTxClass::Begin);
    }

    #[test]
    fn start_alone_is_not_begin() {
        // "START FROM ..." is not a real SQL but ensures we require TRANSACTION token
        assert_eq!(classify_explicit_tx("START FROM foo"), ExplicitTxClass::None);
        assert_eq!(classify_explicit_tx("START"), ExplicitTxClass::None);
    }

    #[test]
    fn commit_variants() {
        assert_eq!(classify_explicit_tx("COMMIT"), ExplicitTxClass::Commit);
        assert_eq!(classify_explicit_tx("commit;"), ExplicitTxClass::Commit);
        assert_eq!(classify_explicit_tx("END"), ExplicitTxClass::Commit);
        assert_eq!(classify_explicit_tx("END WORK"), ExplicitTxClass::Commit);
    }

    #[test]
    fn rollback_variants() {
        assert_eq!(classify_explicit_tx("ROLLBACK"), ExplicitTxClass::Rollback);
        assert_eq!(classify_explicit_tx("rollback;"), ExplicitTxClass::Rollback);
        assert_eq!(classify_explicit_tx("ABORT"), ExplicitTxClass::Rollback);
        assert_eq!(classify_explicit_tx("ABORT WORK"), ExplicitTxClass::Rollback);
    }

    #[test]
    fn savepoint_classified() {
        assert_eq!(classify_explicit_tx("SAVEPOINT sp1"), ExplicitTxClass::Savepoint);
        assert_eq!(classify_explicit_tx("savepoint foo"), ExplicitTxClass::Savepoint);
    }

    #[test]
    fn other_sql_returns_none() {
        assert_eq!(classify_explicit_tx("SELECT 1"), ExplicitTxClass::None);
        assert_eq!(classify_explicit_tx("INSERT INTO t VALUES (1)"), ExplicitTxClass::None);
        assert_eq!(classify_explicit_tx(""), ExplicitTxClass::None);
        assert_eq!(classify_explicit_tx("   "), ExplicitTxClass::None);
    }

    #[test]
    fn leading_whitespace_ignored() {
        assert_eq!(classify_explicit_tx("   BEGIN"), ExplicitTxClass::Begin);
        assert_eq!(classify_explicit_tx("\n\t  COMMIT"), ExplicitTxClass::Commit);
    }

    #[test]
    fn line_comment_skipped() {
        assert_eq!(
            classify_explicit_tx("-- start a tx\nBEGIN"),
            ExplicitTxClass::Begin
        );
        assert_eq!(
            classify_explicit_tx("-- one\n-- two\nROLLBACK"),
            ExplicitTxClass::Rollback
        );
    }

    #[test]
    fn block_comment_skipped() {
        assert_eq!(
            classify_explicit_tx("/* a */ BEGIN"),
            ExplicitTxClass::Begin
        );
        assert_eq!(
            classify_explicit_tx("/* multi\nline */ COMMIT"),
            ExplicitTxClass::Commit
        );
    }

    #[test]
    fn unterminated_block_comment_returns_none() {
        assert_eq!(classify_explicit_tx("/* unterminated"), ExplicitTxClass::None);
    }

    #[test]
    fn is_explicit_begin_alias_matches_class() {
        assert!(is_explicit_begin("BEGIN"));
        assert!(is_explicit_begin("  begin transaction"));
        assert!(!is_explicit_begin("COMMIT"));
        assert!(!is_explicit_begin("SELECT 1"));
    }

    #[test]
    fn is_explicit_tx_end_covers_commit_and_rollback() {
        assert!(is_explicit_tx_end("COMMIT"));
        assert!(is_explicit_tx_end("end"));
        assert!(is_explicit_tx_end("ROLLBACK"));
        assert!(is_explicit_tx_end("abort"));
        assert!(!is_explicit_tx_end("BEGIN"));
        assert!(!is_explicit_tx_end("SELECT 1"));
    }

    #[test]
    fn select_followed_by_begin_keyword_in_string_is_none() {
        // BEGIN appears mid-statement, not first keyword — should NOT classify
        assert_eq!(
            classify_explicit_tx("SELECT 'BEGIN' AS x"),
            ExplicitTxClass::None
        );
    }
}
