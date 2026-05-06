use crate::db::schema_diff::SchemaDiff;
use crate::types::ConnectionId;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStep {
    SelectConnections,
    DiffResult,
    SqlPreview,
    Applying,
    Complete,
}

#[allow(dead_code)]
pub struct MigrationWizardState {
    pub show: bool,
    pub step: MigrationStep,
    pub source_conn: Option<ConnectionId>,
    pub target_conn: Option<ConnectionId>,
    pub source_schema: String,
    pub target_schema: String,
    pub diff: Option<SchemaDiff>,
    pub selected_changes: HashSet<String>,
    pub generated_sql: Option<String>,
    pub applying: bool,
    pub apply_error: Option<String>,
    pub apply_success: bool,
    pub loading_diff: bool,
}

impl Default for MigrationWizardState {
    fn default() -> Self {
        Self {
            show: false,
            step: MigrationStep::SelectConnections,
            source_conn: None,
            target_conn: None,
            source_schema: String::new(),
            target_schema: String::new(),
            diff: None,
            selected_changes: HashSet::new(),
            generated_sql: None,
            applying: false,
            apply_error: None,
            apply_success: false,
            loading_diff: false,
        }
    }
}

impl MigrationWizardState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn open(&mut self) {
        self.reset();
        self.show = true;
    }

    pub fn go_to(&mut self, step: MigrationStep) {
        self.step = step;
    }

    pub fn can_compare(&self) -> bool {
        self.source_conn.is_some()
            && self.target_conn.is_some()
            && !self.source_schema.is_empty()
            && !self.target_schema.is_empty()
            && !self.loading_diff
    }
}
