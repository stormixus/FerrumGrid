//! 스키마(테이블·컬럼) 기반으로 무작위 INSERT SQL 문자열을 생성.
//!
//! 외부 `rand` 의존성 없이 LCG 난수를 사용한다. 생성되는 값은 단순하지만
//! 개발/QA 환경에서 빠르게 더미 데이터를 넣을 때 유용하다.

use crate::types::ColumnInfo;

pub fn generate_inserts(
    schema: &str,
    table: &str,
    columns: &[ColumnInfo],
    row_count: usize,
) -> String {
    let mut rng_state: u64 = 0x1234_5678_DEAD_BEEF;
    let mut gen = || -> f64 {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
            ((rng_state >> 33) as u32 as f64) / (u32::MAX as f64)
        };

    let mut sql = String::new();
    sql.push_str(&format!(
        "-- {} rows of random data for {}.{}\n",
        row_count, schema, table
    ));
    for _ in 0..row_count {
        sql.push_str(&format!("INSERT INTO {}.{} (", schema, table));
        let col_names: Vec<String> = columns
            .iter()
            .map(|c| format!("\"{}\"", c.name))
            .collect();
        sql.push_str(&col_names.join(", "));
        sql.push_str(") VALUES (");
        let values: Vec<String> = columns
            .iter()
            .map(|c| random_value(&c.data_type, &mut gen))
            .collect();
        sql.push_str(&values.join(", "));
        sql.push_str(");\n");
    }
    sql
}

fn random_value(data_type: &str, rng: &mut dyn FnMut() -> f64) -> String {
    let r = rng();
    let dt = data_type.to_lowercase();
    if dt.contains("bool") {
        if r < 0.5 { "TRUE" } else { "FALSE" }.to_string()
    } else if dt.contains("int") || dt.contains("serial") {
        format!("{}", (r * 1000.0) as i64)
    } else if dt.contains("numeric") || dt.contains("decimal") || dt.contains("float") || dt.contains("double") {
        format!("{:.2}", r * 1000.0)
    } else if dt.contains("uuid") {
        "gen_random_uuid()".to_string()
    } else if dt.contains("date") || dt.contains("time") {
        "'2024-01-01 00:00:00'".to_string()
    } else if dt.contains("json") {
        "'{}'::jsonb".to_string()
    } else {
        format!("'sample_{}'", (r * 1000.0) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_row_count() {
        let cols = vec![
            ColumnInfo { name: "id".into(), data_type: "integer".into(), is_nullable: false, is_primary_key: true, default_value: None, comment: None, enum_values: vec![] },
            ColumnInfo { name: "name".into(), data_type: "text".into(), is_nullable: true, is_primary_key: false, default_value: None, comment: None, enum_values: vec![] },
        ];
        let sql = generate_inserts("public", "users", &cols, 3);
        assert_eq!(sql.matches("INSERT INTO").count(), 3);
        assert!(sql.contains("public.users"));
    }
}