/// Defaults copied from the Metal toolchain on macOS 26 / Xcode 26. AIR is
/// versioned together with the SDK, so these may need updating.
pub const AIR_DATALAYOUT: &str = "e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v24:32:32-v32:32:32-v48:64:64-v64:64:64-v96:128:128-v128:128:128-v192:256:256-v256:256:256-v512:512:512-v1024:1024:1024-n8:16:32";
pub const AIR_TRIPLE: &str = "air64_v28-apple-macosx26.0.0";
pub const AIR_VERSION: (u32, u32, u32) = (2, 8, 0);
pub const METAL_VERSION: (u32, u32, u32) = (4, 0, 0);
