pub mod eval;
pub mod expr;
pub mod parser;
pub mod walker;

// Re-export old cli module name for backward compatibility with tests
pub mod cli {
    pub use crate::expr::Config;
}
