use crate::KernelArg;

/// Reflection metadata for one kernel argument.
pub fn air_arg_metadata(index: usize, arg: &KernelArg) -> String {
    match arg {
        KernelArg::ThreadId => format!(
            "!{{i32 {index}, !\"air.thread_position_in_grid\", !\"air.arg_type_name\", !\"uint\", !\"air.arg_name\", !\"tid\"}}"
        ),
        KernelArg::Buffer {
            name,
            llvm_ty,
            size,
            align,
            writable,
            space,
        } => {
            let access = if *writable {
                "air.read_write"
            } else {
                "air.read"
            };
            let sized = if *space == 2 {
                format!("!\"air.buffer_size\", i32 {}, ", size * 3)
            } else {
                String::new()
            };
            format!(
                "!{{i32 {index}, !\"air.buffer\", {sized}!\"air.location_index\", i32 {index}, i32 1, !\"{access}\", !\"air.address_space\", i32 {space}, !\"air.arg_type_size\", i32 {size}, !\"air.arg_type_align_size\", i32 {align}, !\"air.arg_type_name\", !\"{llvm_ty}\", !\"air.arg_name\", !\"{name}\"}}"
            )
        }
    }
}
