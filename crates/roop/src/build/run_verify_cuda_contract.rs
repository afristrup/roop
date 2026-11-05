use crate::{CliError, VerifyCudaContractArgs};

pub fn run_verify_cuda_contract(args: &VerifyCudaContractArgs) -> Result<(), CliError> {
    let ptx = std::fs::read_to_string(&args.ptx)
        .map_err(|e| CliError::Io(args.ptx.display().to_string(), e))?;
    let text = std::fs::read_to_string(&args.contract)
        .map_err(|e| CliError::Io(args.contract.display().to_string(), e))?;
    let contract: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| CliError::Tool(format!("{}: {e}", args.contract.display())))?;
    if contract["schema"] != "roop.cuda-contract.v2" {
        return Err(CliError::Tool("expected roop.cuda-contract.v2".into()));
    }
    let kernels = contract["kernels"]
        .as_array()
        .ok_or_else(|| CliError::Tool("contract kernels must be an array".into()))?;
    for kernel in kernels {
        let name = kernel["kernel"]
            .as_str()
            .ok_or_else(|| CliError::Tool("contract kernel name must be a string".into()))?;
        let header = ptx
            .lines()
            .find(|line| line.contains(&format!("define ptx_kernel void @{name}")))
            .ok_or_else(|| CliError::Tool(format!("PTX has no kernel `{name}`")))?;
        let buffers = kernel["buffers"]
            .as_array()
            .ok_or_else(|| CliError::Tool(format!("kernel `{name}` has no buffer array")))?;
        for buffer in buffers {
            let role = buffer["role"].as_str().ok_or_else(|| {
                CliError::Tool(format!("kernel `{name}` has a buffer without a role"))
            })?;
            let parameter = match role {
                "capture" => format!("%buf{}", buffer["slot"]),
                "launch_params" => "%params".into(),
                "error" => "%err".into(),
                _ => {
                    return Err(CliError::Tool(format!(
                        "kernel `{name}` has unknown buffer role `{role}`"
                    )));
                }
            };
            let space = buffer["address_space"].as_u64().ok_or_else(|| {
                CliError::Tool(format!(
                    "kernel `{name}` has a buffer without an address space"
                ))
            })?;
            let pointer = format!("ptr addrspace({space}) {parameter}");
            if !header.contains(&pointer) {
                return Err(CliError::Tool(format!(
                    "kernel `{name}` is missing `{pointer}`"
                )));
            }
        }
    }
    println!("CUDA contract matches PTX LLVM signatures");
    Ok(())
}
