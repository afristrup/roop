use super::{Ctx, Slot};

/// Per-function emission state. Allocas are hoisted to the entry block.
pub struct FnGen<'a> {
    pub ctx: &'a Ctx<'a>,
    pub allocas: String,
    pub body: String,
    pub vars: Vec<(String, Slot)>,
    next: usize,
}

impl<'a> FnGen<'a> {
    pub fn new(ctx: &'a Ctx<'a>) -> Self {
        FnGen {
            ctx,
            allocas: String::new(),
            body: String::new(),
            vars: Vec::new(),
            next: 0,
        }
    }

    pub fn fresh(&mut self, prefix: &str) -> String {
        self.next += 1;
        format!("{prefix}{}", self.next)
    }

    pub fn emit(&mut self, line: &str) {
        self.body.push_str("  ");
        self.body.push_str(line);
        self.body.push('\n');
    }

    pub fn label(&mut self, name: &str) {
        self.body.push_str(name);
        self.body.push_str(":\n");
    }

    pub fn alloca(&mut self, llvm_ty: &str) -> String {
        let addr = format!("%{}", self.fresh("a"));
        self.allocas.push_str(&format!("  {addr} = alloca {llvm_ty}\n"));
        addr
    }
}
