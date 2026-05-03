pub mod cli;
pub mod parser;
pub mod sync;
pub mod ui;

pub use cli::{
    append_model_to_schema, check_prisma_installed, generate_schema_file, get_prisma_version,
    run_prisma_cli, PrismaCommand,
};
pub use parser::PrismaSchema;
pub use sync::{generate_migration, sync_db_to_schema, sync_schema_to_db};
