use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;
use crate::types::ForeignKey;

pub async fn list_foreign_keys(
    client: &Client,
    schema: &str,
    conn_id: ConnectionId,
) -> Result<Vec<ForeignKey>, DbError> {
    let rows = client
        .query(
            "SELECT
                tc.constraint_name,
                tc.table_schema,
                tc.table_name,
                kcu.column_name,
                ccu.table_schema AS foreign_table_schema,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
            FROM
                information_schema.table_constraints AS tc
                JOIN information_schema.key_column_usage AS kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage AS ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
            WHERE
                tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = $1",
            &[&schema],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| ForeignKey {
            name: r.get(0),
            source_schema: r.get(1),
            source_table: r.get(2),
            source_column: r.get(3),
            target_schema: r.get(4),
            target_table: r.get(5),
            target_column: r.get(6),
        })
        .collect())
}
