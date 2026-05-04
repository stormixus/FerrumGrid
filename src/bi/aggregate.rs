//! BI 카드 의 column 통계 + group-by 계산.
//!
//! Plan v7 Phase 4c1 — backend helpers. UI 측 (`src/ui/objects/bi.rs`) 의
//! inline 계산을 추출하여 단위 테스트 가능 형태로.

use crate::types::{CellValue, QueryResult};

/// 집계 연산자.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateOp {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// 단일 column 의 요약 통계 (BI 카드 의 row 1개).
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnStats {
    pub name: String,
    pub type_name: String,
    /// non-null count.
    pub non_null: usize,
    /// numeric 값들의 min — 비-numeric 또는 빈 결과 시 None.
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub avg: Option<f64>,
}

/// `QueryResult` 의 col_idx 컬럼에 대한 통계 계산.
///
/// numeric 셀 (Int / Float) 은 min/max/avg 에 누적. 다른 타입 셀은 non_null 만
/// 증가. Null 은 모두 무시.
pub fn compute_column_stats(result: &QueryResult, col_idx: usize) -> Option<ColumnStats> {
    let column = result.columns.get(col_idx)?;
    let mut non_null = 0usize;
    let mut numeric_values: Vec<f64> = Vec::new();

    for row in &result.rows {
        if let Some(cell) = row.get(col_idx) {
            match cell {
                CellValue::Int(v) => {
                    non_null += 1;
                    numeric_values.push(*v as f64);
                }
                CellValue::Float(v) => {
                    non_null += 1;
                    numeric_values.push(*v);
                }
                CellValue::Null => {}
                _ => non_null += 1,
            }
        }
    }

    let (min, max, avg) = if numeric_values.is_empty() {
        (None, None, None)
    } else {
        let min = numeric_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = numeric_values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let avg = numeric_values.iter().sum::<f64>() / numeric_values.len() as f64;
        (Some(min), Some(max), Some(avg))
    };

    Some(ColumnStats {
        name: column.name.clone(),
        type_name: column.type_name.clone(),
        non_null,
        min,
        max,
        avg,
    })
}

/// `QueryResult` 의 모든 column 에 대해 통계 계산.
#[allow(dead_code)]
pub fn compute_all_stats(result: &QueryResult) -> Vec<ColumnStats> {
    (0..result.columns.len())
        .filter_map(|idx| compute_column_stats(result, idx))
        .collect()
}

/// Group-by 집계: `group_col_idx` 의 셀을 키로 사용하여 `agg_col_idx` 컬럼을
/// `op` 로 집계. Null 그룹 키는 무시.
///
/// 결과 순서: group_key 의 *알파벳* 정렬 (BI 표시 안정성).
#[allow(dead_code)]
pub fn group_by(
    result: &QueryResult,
    group_col_idx: usize,
    agg_col_idx: usize,
    op: AggregateOp,
) -> Vec<(String, f64)> {
    use std::collections::BTreeMap;

    let mut buckets: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for row in &result.rows {
        let group_key = match row.get(group_col_idx) {
            Some(CellValue::Null) | None => continue,
            Some(cell) => cell_to_group_key(cell),
        };
        let value = match row.get(agg_col_idx) {
            Some(CellValue::Int(v)) => *v as f64,
            Some(CellValue::Float(v)) => *v,
            _ => {
                // Count 는 non-numeric 도 1 카운트 (null 제외)
                if matches!(op, AggregateOp::Count) {
                    if !matches!(row.get(agg_col_idx), Some(CellValue::Null) | None) {
                        buckets.entry(group_key).or_default().push(1.0);
                    }
                    continue;
                } else {
                    continue;
                }
            }
        };
        buckets.entry(group_key).or_default().push(value);
    }

    buckets
        .into_iter()
        .map(|(key, values)| {
            let agg = match op {
                AggregateOp::Count => values.len() as f64,
                AggregateOp::Sum => values.iter().sum(),
                AggregateOp::Avg => {
                    if values.is_empty() {
                        0.0
                    } else {
                        values.iter().sum::<f64>() / values.len() as f64
                    }
                }
                AggregateOp::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
                AggregateOp::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            };
            (key, agg)
        })
        .collect()
}

fn cell_to_group_key(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => "(null)".to_string(),
        CellValue::Bool(v) => v.to_string(),
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => format!("{v:?}"),
        CellValue::Text(s) | CellValue::Timestamp(s) | CellValue::Unknown(s) => s.clone(),
        CellValue::Json(v) => v.to_string(),
        CellValue::Uuid(u) => u.to_string(),
        CellValue::Bytes(_) => "(bytes)".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnMeta;

    fn make_result(columns: Vec<(&str, &str)>, rows: Vec<Vec<CellValue>>) -> QueryResult {
        QueryResult {
            columns: columns
                .into_iter()
                .map(|(name, ty)| ColumnMeta {
                    name: name.to_string(),
                    type_name: ty.to_string(),
                })
                .collect(),
            rows,
            execution_time_ms: 0,
        }
    }

    #[test]
    fn stats_for_int_column_includes_min_max_avg() {
        let result = make_result(
            vec![("price", "int4")],
            vec![
                vec![CellValue::Int(10)],
                vec![CellValue::Int(20)],
                vec![CellValue::Int(30)],
            ],
        );
        let stats = compute_column_stats(&result, 0).unwrap();
        assert_eq!(stats.non_null, 3);
        assert_eq!(stats.min, Some(10.0));
        assert_eq!(stats.max, Some(30.0));
        assert_eq!(stats.avg, Some(20.0));
    }

    #[test]
    fn stats_for_text_column_only_non_null() {
        let result = make_result(
            vec![("name", "text")],
            vec![
                vec![CellValue::Text("a".to_string())],
                vec![CellValue::Text("b".to_string())],
                vec![CellValue::Null],
            ],
        );
        let stats = compute_column_stats(&result, 0).unwrap();
        assert_eq!(stats.non_null, 2);
        assert!(stats.min.is_none());
        assert!(stats.max.is_none());
        assert!(stats.avg.is_none());
    }

    #[test]
    fn stats_for_empty_result_returns_zero_non_null() {
        let result = make_result(vec![("price", "int4")], vec![]);
        let stats = compute_column_stats(&result, 0).unwrap();
        assert_eq!(stats.non_null, 0);
        assert!(stats.min.is_none());
    }

    #[test]
    fn stats_for_invalid_col_idx_returns_none() {
        let result = make_result(vec![("price", "int4")], vec![]);
        assert!(compute_column_stats(&result, 99).is_none());
    }

    #[test]
    fn stats_skips_null_in_numeric() {
        let result = make_result(
            vec![("price", "numeric")],
            vec![
                vec![CellValue::Int(5)],
                vec![CellValue::Null],
                vec![CellValue::Int(15)],
            ],
        );
        let stats = compute_column_stats(&result, 0).unwrap();
        assert_eq!(stats.non_null, 2);
        assert_eq!(stats.avg, Some(10.0));
    }

    #[test]
    fn compute_all_stats_returns_one_per_column() {
        let result = make_result(
            vec![("a", "int4"), ("b", "text")],
            vec![vec![CellValue::Int(1), CellValue::Text("x".to_string())]],
        );
        let all = compute_all_stats(&result);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "a");
        assert_eq!(all[1].name, "b");
    }

    #[test]
    fn group_by_count_groups_by_text_column() {
        let result = make_result(
            vec![("category", "text"), ("price", "int4")],
            vec![
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(10),
                ],
                vec![
                    CellValue::Text("b".to_string()),
                    CellValue::Int(20),
                ],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(30),
                ],
            ],
        );
        let groups = group_by(&result, 0, 1, AggregateOp::Count);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], ("a".to_string(), 2.0));
        assert_eq!(groups[1], ("b".to_string(), 1.0));
    }

    #[test]
    fn group_by_sum_aggregates_values() {
        let result = make_result(
            vec![("category", "text"), ("price", "int4")],
            vec![
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(10),
                ],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(30),
                ],
            ],
        );
        let groups = group_by(&result, 0, 1, AggregateOp::Sum);
        assert_eq!(groups[0], ("a".to_string(), 40.0));
    }

    #[test]
    fn group_by_avg_returns_mean() {
        let result = make_result(
            vec![("category", "text"), ("price", "int4")],
            vec![
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(10),
                ],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(30),
                ],
            ],
        );
        let groups = group_by(&result, 0, 1, AggregateOp::Avg);
        assert_eq!(groups[0], ("a".to_string(), 20.0));
    }

    #[test]
    fn group_by_min_max_extremes() {
        let result = make_result(
            vec![("category", "text"), ("price", "int4")],
            vec![
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(50),
                ],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(10),
                ],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(30),
                ],
            ],
        );
        let mins = group_by(&result, 0, 1, AggregateOp::Min);
        assert_eq!(mins[0].1, 10.0);
        let maxes = group_by(&result, 0, 1, AggregateOp::Max);
        assert_eq!(maxes[0].1, 50.0);
    }

    #[test]
    fn group_by_skips_null_keys() {
        let result = make_result(
            vec![("category", "text"), ("price", "int4")],
            vec![
                vec![CellValue::Null, CellValue::Int(10)],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(20),
                ],
            ],
        );
        let groups = group_by(&result, 0, 1, AggregateOp::Count);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "a");
    }

    #[test]
    fn group_by_results_sorted_by_key() {
        let result = make_result(
            vec![("category", "text"), ("price", "int4")],
            vec![
                vec![
                    CellValue::Text("z".to_string()),
                    CellValue::Int(1),
                ],
                vec![
                    CellValue::Text("a".to_string()),
                    CellValue::Int(1),
                ],
                vec![
                    CellValue::Text("m".to_string()),
                    CellValue::Int(1),
                ],
            ],
        );
        let groups = group_by(&result, 0, 1, AggregateOp::Sum);
        let keys: Vec<&str> = groups.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }
}
