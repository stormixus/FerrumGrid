pub mod cli;
pub mod parser;
pub mod sync;
pub mod ui;

pub use parser::PrismaSchema;
pub use cli::{PrismaCommand, run_prisma_cli, check_prisma_installed, generate_schema_file};
