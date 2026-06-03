//! 1급 카탈로그 객체 조회 — 시퀀스 / 사용자 정의 enum 타입 / 익스텐션.
//! (기존 객체 브라우저는 테이블/뷰/함수/롤만 노출하므로 이들을 보완.)

use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;

#[derive(Debug, Clone)]
pub struct SequenceInfo {
    pub schema: String,
    pub name: String,
    pub data_type: String,
    pub start_value: String,
    pub increment: String,
}

#[derive(Debug, Clone)]
pub struct EnumTypeInfo {
    pub schema: String,
    pub name: String,
    pub labels: String,
}

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Default)]
pub struct CatalogObjects {
    pub sequences: Vec<SequenceInfo>,
    pub enums: Vec<EnumTypeInfo>,
    pub extensions: Vec<ExtensionInfo>,
}

pub async fn list_catalog(
    client: &Client,
    conn_id: ConnectionId,
) -> Result<CatalogObjects, DbError> {
    let seq_rows = client
        .query(
            "SELECT sequence_schema, sequence_name, data_type, \
                    start_value::text, increment::text \
             FROM information_schema.sequences ORDER BY 1, 2",
            &[],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let sequences = seq_rows
        .iter()
        .map(|r| SequenceInfo {
            schema: r.get(0),
            name: r.get(1),
            data_type: r.get(2),
            start_value: r.get(3),
            increment: r.get(4),
        })
        .collect();

    let enum_rows = client
        .query(
            "SELECT n.nspname, t.typname, \
                    string_agg(e.enumlabel, ', ' ORDER BY e.enumsortorder) \
             FROM pg_catalog.pg_type t \
             JOIN pg_catalog.pg_enum e ON e.enumtypid = t.oid \
             JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace \
             GROUP BY 1, 2 ORDER BY 1, 2",
            &[],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let enums = enum_rows
        .iter()
        .map(|r| EnumTypeInfo {
            schema: r.get(0),
            name: r.get(1),
            labels: r.get(2),
        })
        .collect();

    let ext_rows = client
        .query(
            "SELECT extname, COALESCE(extversion, '') FROM pg_catalog.pg_extension ORDER BY extname",
            &[],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let extensions = ext_rows
        .iter()
        .map(|r| ExtensionInfo {
            name: r.get(0),
            version: r.get(1),
        })
        .collect();

    Ok(CatalogObjects {
        sequences,
        enums,
        extensions,
    })
}
