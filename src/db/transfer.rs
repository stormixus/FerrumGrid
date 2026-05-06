use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use crate::types::ForeignKey;

/// Error types for cross-server table transfer operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TransferError {
    /// Tables form a circular dependency. Contains the cycle.
    CircularDependency(Vec<(String, String)>),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::CircularDependency(cycle) => {
                write!(f, "circular dependency: ")?;
                for (i, (schema, table)) in cycle.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{schema}.{table}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for TransferError {}

/// Return tables in dependency order (dependencies first) using Kahn's algorithm.
///
/// Tables that appear in `tables` but have no foreign-key relationships are
/// included in the output (they can appear in any position before their
/// dependents).
#[allow(dead_code)]
pub fn dependency_order(
    tables: &[(String, String)],
    foreign_keys: &[ForeignKey],
) -> Result<Vec<(String, String)>, TransferError> {
    let table_set: HashSet<(String, String)> = tables.iter().cloned().collect();

    // Build adjacency list and in-degree map.
    // An FK from source -> target means source *depends on* target,
    // so target must come first. Edge: target -> source in topo order.
    let mut in_degree: HashMap<(String, String), usize> = HashMap::new();
    let mut adjacency: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();

    // Initialize all tables with zero in-degree.
    for t in &table_set {
        in_degree.entry(t.clone()).or_insert(0);
        adjacency.entry(t.clone()).or_default();
    }

    // Process foreign keys: source depends on target.
    for fk in foreign_keys {
        let source = (fk.source_schema.clone(), fk.source_table.clone());
        let target = (fk.target_schema.clone(), fk.target_table.clone());

        // Only consider FKs where both ends are in the requested table set.
        if !table_set.contains(&source) || !table_set.contains(&target) {
            continue;
        }
        // Skip self-references.
        if source == target {
            continue;
        }

        // target -> source edge (target must come before source).
        adjacency.entry(target.clone()).or_default().push(source.clone());
        *in_degree.entry(source).or_insert(0) += 1;
    }

    // Kahn's algorithm.
    let mut queue: VecDeque<(String, String)> = VecDeque::new();
    for (table, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(table.clone());
        }
    }

    let mut result: Vec<(String, String)> = Vec::with_capacity(tables.len());

    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&node) {
            for neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        result.push(node);
    }

    if result.len() != table_set.len() {
        // Circular dependency detected. Extract the cycle from remaining nodes.
        let remaining: Vec<(String, String)> = in_degree
            .into_iter()
            .filter(|(_, deg)| *deg > 0)
            .map(|(t, _)| t)
            .collect();
        return Err(TransferError::CircularDependency(remaining));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ForeignKey;

    fn fk(
        src_schema: &str,
        src_table: &str,
        src_col: &str,
        tgt_schema: &str,
        tgt_table: &str,
        tgt_col: &str,
    ) -> ForeignKey {
        ForeignKey {
            name: format!("fk_{src_table}_{tgt_table}"),
            source_schema: src_schema.to_string(),
            source_table: src_table.to_string(),
            source_column: src_col.to_string(),
            target_schema: tgt_schema.to_string(),
            target_table: tgt_table.to_string(),
            target_column: tgt_col.to_string(),
        }
    }

    #[test]
    fn linear_chain_dependencies_first() {
        // A depends on B, B depends on C => order: C, B, A
        let tables = vec![
            ("public".into(), "a".into()),
            ("public".into(), "b".into()),
            ("public".into(), "c".into()),
        ];
        let fks = vec![
            fk("public", "a", "b_id", "public", "b", "id"),
            fk("public", "b", "c_id", "public", "c", "id"),
        ];

        let result = dependency_order(&tables, &fks).unwrap();

        // C must come before B, B before A.
        let pos = |name: &str| {
            result
                .iter()
                .position(|(_, t)| t == name)
                .unwrap_or_else(|| panic!("{name} not found"))
        };
        assert!(pos("c") < pos("b"));
        assert!(pos("b") < pos("a"));
    }

    #[test]
    fn diamond_dependency() {
        // A depends on B and C; B and C both depend on D.
        let tables = vec![
            ("public".into(), "a".into()),
            ("public".into(), "b".into()),
            ("public".into(), "c".into()),
            ("public".into(), "d".into()),
        ];
        let fks = vec![
            fk("public", "a", "b_id", "public", "b", "id"),
            fk("public", "a", "c_id", "public", "c", "id"),
            fk("public", "b", "d_id", "public", "d", "id"),
            fk("public", "c", "d_id", "public", "d", "id"),
        ];

        let result = dependency_order(&tables, &fks).unwrap();

        let pos = |name: &str| {
            result
                .iter()
                .position(|(_, t)| t == name)
                .unwrap_or_else(|| panic!("{name} not found"))
        };
        assert!(pos("d") < pos("b"));
        assert!(pos("d") < pos("c"));
        assert!(pos("b") < pos("a"));
        assert!(pos("c") < pos("a"));
    }

    #[test]
    fn independent_tables_all_included() {
        let tables = vec![
            ("public".into(), "x".into()),
            ("public".into(), "y".into()),
            ("public".into(), "z".into()),
        ];
        let fks: Vec<ForeignKey> = vec![];

        let result = dependency_order(&tables, &fks).unwrap();
        assert_eq!(result.len(), 3);

        let names: HashSet<String> = result.into_iter().map(|(_, t)| t).collect();
        assert!(names.contains("x"));
        assert!(names.contains("y"));
        assert!(names.contains("z"));
    }

    #[test]
    fn circular_dependency_detected() {
        // A -> B -> A
        let tables = vec![
            ("public".into(), "a".into()),
            ("public".into(), "b".into()),
        ];
        let fks = vec![
            fk("public", "a", "b_id", "public", "b", "id"),
            fk("public", "b", "a_id", "public", "a", "id"),
        ];

        let err = dependency_order(&tables, &fks).unwrap_err();
        match err {
            TransferError::CircularDependency(cycle) => {
                assert_eq!(cycle.len(), 2);
                let names: HashSet<String> = cycle.into_iter().map(|(_, t)| t).collect();
                assert!(names.contains("a"));
                assert!(names.contains("b"));
            }
        }
    }

    #[test]
    fn tables_without_fk_deps_included() {
        // Only A depends on B; C has no FK relations.
        let tables = vec![
            ("public".into(), "a".into()),
            ("public".into(), "b".into()),
            ("public".into(), "c".into()),
        ];
        let fks = vec![fk("public", "a", "b_id", "public", "b", "id")];

        let result = dependency_order(&tables, &fks).unwrap();
        assert_eq!(result.len(), 3);

        let pos = |name: &str| {
            result
                .iter()
                .position(|(_, t)| t == name)
                .unwrap_or_else(|| panic!("{name} not found"))
        };
        assert!(pos("b") < pos("a"));
        // c can be anywhere — just verify it's present.
        let _ = pos("c");
    }

    #[test]
    fn cross_schema_foreign_keys() {
        // public.orders depends on auth.users
        let tables = vec![
            ("public".into(), "orders".into()),
            ("auth".into(), "users".into()),
        ];
        let fks = vec![fk(
            "public", "orders", "user_id", "auth", "users", "id",
        )];

        let result = dependency_order(&tables, &fks).unwrap();

        let pos = |schema: &str, name: &str| {
            result
                .iter()
                .position(|(s, t)| s == schema && t == name)
                .unwrap_or_else(|| panic!("{schema}.{name} not found"))
        };
        assert!(pos("auth", "users") < pos("public", "orders"));
    }
}
