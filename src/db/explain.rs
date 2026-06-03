//! EXPLAIN (FORMAT JSON) 실행 + 플랜 JSON 파싱.
//!
//! 기본은 `EXPLAIN (FORMAT JSON)` (plan-only) — 쿼리를 실제로 실행하지 않으므로
//! write 문에도 안전하다. (ANALYZE 는 실행을 동반하므로 의도적으로 제외.)

use serde_json::Value;

use crate::db::error::DbError;
use crate::types::ConnectionId;

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNode {
    pub node_type: String,
    pub relation: Option<String>,
    pub total_cost: Option<f64>,
    pub plan_rows: Option<f64>,
    /// ANALYZE 시에만 존재 (estimated vs actual 비교용).
    pub actual_rows: Option<f64>,
    pub children: Vec<PlanNode>,
}

/// 최상위 EXPLAIN JSON (`[ { "Plan": {...} } ]`) 을 PlanNode 트리로 파싱.
pub fn parse_explain_json(json: &str) -> Option<PlanNode> {
    let v: Value = serde_json::from_str(json).ok()?;
    let plan = v
        .get(0)
        .and_then(|o| o.get("Plan"))
        .or_else(|| v.get("Plan"))?;
    Some(parse_node(plan))
}

fn parse_node(v: &Value) -> PlanNode {
    PlanNode {
        node_type: v
            .get("Node Type")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string(),
        relation: v
            .get("Relation Name")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        total_cost: v.get("Total Cost").and_then(|x| x.as_f64()),
        plan_rows: v.get("Plan Rows").and_then(|x| x.as_f64()),
        actual_rows: v.get("Actual Rows").and_then(|x| x.as_f64()),
        children: v
            .get("Plans")
            .and_then(|x| x.as_array())
            .map(|arr| arr.iter().map(parse_node).collect())
            .unwrap_or_default(),
    }
}

/// `EXPLAIN (FORMAT JSON) <sql>` 실행 후 플랜 JSON 문자열 반환 (plan-only).
pub async fn run_explain(
    client: &tokio_postgres::Client,
    sql: &str,
    conn_id: ConnectionId,
) -> Result<String, DbError> {
    let explain_sql = format!("EXPLAIN (FORMAT JSON) {sql}");
    let rows = client
        .query(explain_sql.as_str(), &[])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let row = rows
        .first()
        .ok_or_else(|| DbError::internal(conn_id, "EXPLAIN returned no rows"))?;
    let value: Value = row.get(0);
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_plan() {
        let json = r#"[
          {"Plan": {
            "Node Type": "Hash Join",
            "Total Cost": 123.45,
            "Plan Rows": 100,
            "Plans": [
              {"Node Type": "Seq Scan", "Relation Name": "orders", "Total Cost": 50.0, "Plan Rows": 1000},
              {"Node Type": "Hash", "Total Cost": 20.0, "Plan Rows": 10,
                "Plans": [{"Node Type": "Seq Scan", "Relation Name": "users", "Total Cost": 10.0, "Plan Rows": 10}]}
            ]
          }}
        ]"#;
        let root = parse_explain_json(json).expect("parses");
        assert_eq!(root.node_type, "Hash Join");
        assert_eq!(root.total_cost, Some(123.45));
        assert_eq!(root.children.len(), 2);
        assert_eq!(root.children[0].relation.as_deref(), Some("orders"));
        assert_eq!(root.children[1].children[0].relation.as_deref(), Some("users"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_explain_json("not json").is_none());
        assert!(parse_explain_json("[]").is_none());
    }
}
