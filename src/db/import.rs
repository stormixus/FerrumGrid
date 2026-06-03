//! CSV → 테이블 import (COPY FROM STDIN). 파싱은 Postgres 가 수행하므로
//! 클라이언트에서 CSV 를 파싱할 필요 없이 raw 바이트를 스트리밍한다.
//! 헤더(첫 행) 컬럼명으로 COPY 컬럼 리스트를 구성해 이름 기준 매핑한다.

use futures_util::SinkExt;
use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// CSV 파일의 헤더(첫 행) 컬럼명을 읽는다.
fn read_csv_headers(path: &std::path::Path) -> std::io::Result<Vec<String>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;
    Ok(rdr.headers()?.iter().map(|h| h.to_string()).collect())
}

/// CSV 파일을 대상 테이블로 적재. 헤더명 → 컬럼 매핑 후 `COPY … FROM STDIN
/// WITH (FORMAT csv, HEADER true)` 로 raw 바이트 스트리밍. 적재된 행 수 반환.
pub async fn import_csv_file(
    client: &Client,
    schema: &str,
    table: &str,
    path: &std::path::Path,
    conn_id: ConnectionId,
) -> Result<u64, DbError> {
    let headers = read_csv_headers(path)
        .map_err(|e| DbError::internal(conn_id, format!("Failed to read CSV header: {e}")))?;
    if headers.is_empty() {
        return Err(DbError::internal(conn_id, "CSV file has no header row"));
    }
    let col_list = headers
        .iter()
        .map(|h| quote_ident(h))
        .collect::<Vec<_>>()
        .join(", ");
    let copy_sql = format!(
        "COPY {}.{} ({}) FROM STDIN WITH (FORMAT csv, HEADER true)",
        quote_ident(schema),
        quote_ident(table),
        col_list
    );
    let data = std::fs::read(path)
        .map_err(|e| DbError::internal(conn_id, format!("Failed to read CSV file: {e}")))?;
    let sink = client
        .copy_in(&copy_sql)
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let mut sink = Box::pin(sink);
    sink.as_mut()
        .send(bytes::Bytes::from(data))
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    sink.as_mut()
        .finish()
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_headers_from_csv() {
        let mut f = tempfile_csv("id,name,email\n1,a,a@x.com\n");
        let headers = read_csv_headers(f.path()).unwrap();
        assert_eq!(headers, vec!["id", "name", "email"]);
        f.flush().ok();
    }

    #[test]
    fn quote_ident_escapes_quotes() {
        assert_eq!(quote_ident("a\"b"), "\"a\"\"b\"");
    }

    struct TempCsv {
        path: std::path::PathBuf,
        _file: std::fs::File,
    }
    impl TempCsv {
        fn path(&self) -> &std::path::Path {
            &self.path
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    fn tempfile_csv(contents: &str) -> TempCsv {
        let mut path = std::env::temp_dir();
        path.push(format!("fg_import_test_{}.csv", contents.len()));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        TempCsv { path, _file: file }
    }
}
