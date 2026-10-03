use crate::{AbortMode, Clear, Ctx, Dialect, Kernel, Slot};
use roop_syntax::{Place, Type};

/// Per-function emission state. Allocas are hoisted to the entry block.
pub struct FnGen<'a> {
    pub ctx: &'a Ctx<'a>,
    pub allocas: String,
    pub body: String,
    pub vars: Vec<(String, Slot)>,
    pub symbol: String,
    pub dialect: Dialect,
    /// Device kernels cannot trap; a failed assertion stores 1 here instead.
    pub error_flag: Option<String>,
    pub abort: AbortMode,
    /// The history stacks of the enclosing `logged` blocks, innermost last.
    pub logged: Vec<Place>,
    /// Channels in scope with their message types; each has a `chan:NAME`
    /// variable holding its handle so tasks can capture it.
    pub chan_types: Vec<(String, Type)>,
    pub outlined: Vec<String>,
    pub kernels: Vec<Kernel>,
    /// Module-level definitions (string constants) the function needs.
    pub globals: Vec<String>,
    /// The statements of the enclosing ancillas that are compiled as zeroing.
    pub clears: Vec<Clear>,
    next: usize,
}

impl<'a> FnGen<'a> {
    pub fn new(ctx: &'a Ctx<'a>, symbol: String, dialect: Dialect) -> Self {
        FnGen {
            ctx,
            allocas: String::new(),
            body: String::new(),
            vars: Vec::new(),
            symbol,
            dialect,
            error_flag: None,
            abort: AbortMode::Trap,
            logged: Vec::new(),
            chan_types: Vec::new(),
            outlined: Vec::new(),
            kernels: Vec::new(),
            globals: Vec::new(),
            clears: Vec::new(),
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
        self.allocas
            .push_str(&format!("  {addr} = alloca {llvm_ty}\n"));
        addr
    }
}
