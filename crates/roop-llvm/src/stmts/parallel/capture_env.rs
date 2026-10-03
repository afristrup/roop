use crate::FnGen;

/// Stores the address of every visible variable into an environment array.
pub fn capture_env(g: &mut FnGen) -> String {
    let n = g.vars.len();
    let env = g.alloca(&format!("[{n} x ptr]"));
    let addrs: Vec<String> = g.vars.iter().map(|(_, slot)| slot.addr.clone()).collect();
    for (i, addr) in addrs.iter().enumerate() {
        let at = format!("%{}", g.fresh("t"));
        g.emit(&format!(
            "{at} = getelementptr inbounds [{n} x ptr], ptr {env}, i64 0, i64 {i}"
        ));
        g.emit(&format!("store ptr {addr}, ptr {at}"));
    }
    env
}
