pub mod action;
pub mod bound;
mod stmt;

use action::Action;

pub struct Policy {
    default_action: Action,
    // stmts: Vec<stmt::Stmt>,
}
impl Policy {
    pub fn default_action(mut self, action: Action) -> Self {
        self.default_action = action;
        self
    }
}

#[macro_export]
macro_rules! policy {
    () => {
        todo!()
    };
}
