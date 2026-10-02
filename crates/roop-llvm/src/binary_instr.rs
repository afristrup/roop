use super::{CodegenError, Kind};
use roop_syntax::BinOp;

/// Instruction mnemonic and whether the result is a bool.
pub fn binary_instr(kind: Kind, op: BinOp) -> Result<(&'static str, bool), CodegenError> {
    use BinOp::*;
    use Kind::*;
    let found = match (kind, op) {
        (Int, Add) => ("add", false),
        (Int, Sub) => ("sub", false),
        (Int, Mul) => ("mul", false),
        (Int, Div) => ("sdiv", false),
        (Int, Rem) => ("srem", false),
        (Float, Add) => ("fadd", false),
        (Float, Sub) => ("fsub", false),
        (Float, Mul) => ("fmul", false),
        (Float, Div) => ("fdiv", false),
        (Float, Rem) => ("frem", false),
        (Int | Enum | Bool, Eq) => ("icmp eq", true),
        (Int | Enum | Bool, Ne) => ("icmp ne", true),
        (Int, Lt) => ("icmp slt", true),
        (Int, Le) => ("icmp sle", true),
        (Int, Gt) => ("icmp sgt", true),
        (Int, Ge) => ("icmp sge", true),
        (Float, Eq) => ("fcmp oeq", true),
        (Float, Ne) => ("fcmp une", true),
        (Float, Lt) => ("fcmp olt", true),
        (Float, Le) => ("fcmp ole", true),
        (Float, Gt) => ("fcmp ogt", true),
        (Float, Ge) => ("fcmp oge", true),
        (Bool, And) => ("and", false),
        (Bool, Or) => ("or", false),
        _ => return Err(CodegenError::InvalidOperand("operator not defined for type")),
    };
    Ok(found)
}
