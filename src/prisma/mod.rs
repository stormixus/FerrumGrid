pub mod cli;
pub mod parser;
pub mod sync;
pub mod ui;

pub use cli::{check_prisma_installed, generate_schema_file, run_prisma_cli, PrismaCommand};
pub use parser::PrismaSchema;
