use crate::gpu::{KERNEL_ASSERTION_FAILED, OK, RoopBuf, UNAVAILABLE};
use libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_void};

type Status = i32;
type Ptr = *mut c_void;

fn open_driver() -> Option<Library> {
    ["libcuda.so.1", "libcuda.so", "nvcuda.dll"]
        .into_iter()
        .find_map(|name| unsafe { Library::new(name).ok() })
}

/// Runs a PTX kernel through the CUDA driver API. The driver is loaded at run
/// time, so this builds everywhere and reports `UNAVAILABLE` without CUDA.
/// Written from the driver API reference; not yet exercised on NVIDIA hardware.
pub fn cuda_dispatch(ptx: &[u8], kernel: &str, bufs: &[RoopBuf], params: [i64; 3]) -> i32 {
    let Some(lib) = open_driver() else {
        return UNAVAILABLE;
    };
    unsafe { launch(&lib, ptx, kernel, bufs, params) }.unwrap_or(UNAVAILABLE)
}

macro_rules! driver {
    ($lib:expr, $name:literal, $ty:ty) => {{
        let symbol: Symbol<$ty> = $lib.get(concat!($name, "\0").as_bytes()).ok()?;
        *symbol
    }};
}

fn ok(status: Status) -> Option<()> {
    (status == 0).then_some(())
}

unsafe fn launch(
    lib: &Library,
    ptx: &[u8],
    kernel: &str,
    bufs: &[RoopBuf],
    params: [i64; 3],
) -> Option<i32> {
    unsafe {
        let init = driver!(lib, "cuInit", unsafe extern "C" fn(u32) -> Status);
        let device_get = driver!(
            lib,
            "cuDeviceGet",
            unsafe extern "C" fn(*mut i32, i32) -> Status
        );
        let retain = driver!(
            lib,
            "cuDevicePrimaryCtxRetain",
            unsafe extern "C" fn(*mut Ptr, i32) -> Status
        );
        let set_ctx = driver!(lib, "cuCtxSetCurrent", unsafe extern "C" fn(Ptr) -> Status);
        let load = driver!(
            lib,
            "cuModuleLoadData",
            unsafe extern "C" fn(*mut Ptr, *const c_void) -> Status
        );
        let get_fn = driver!(
            lib,
            "cuModuleGetFunction",
            unsafe extern "C" fn(*mut Ptr, Ptr, *const c_char) -> Status
        );
        let alloc = driver!(
            lib,
            "cuMemAlloc_v2",
            unsafe extern "C" fn(*mut u64, usize) -> Status
        );
        let to_dev = driver!(
            lib,
            "cuMemcpyHtoD_v2",
            unsafe extern "C" fn(u64, *const c_void, usize) -> Status
        );
        let to_host = driver!(
            lib,
            "cuMemcpyDtoH_v2",
            unsafe extern "C" fn(*mut c_void, u64, usize) -> Status
        );
        let free = driver!(lib, "cuMemFree_v2", unsafe extern "C" fn(u64) -> Status);
        let launch = driver!(
            lib,
            "cuLaunchKernel",
            unsafe extern "C" fn(
                Ptr,
                u32,
                u32,
                u32,
                u32,
                u32,
                u32,
                u32,
                Ptr,
                *mut Ptr,
                *mut Ptr,
            ) -> Status
        );
        let sync = driver!(lib, "cuCtxSynchronize", unsafe extern "C" fn() -> Status);

        ok(init(0))?;
        let mut device = 0;
        ok(device_get(&mut device, 0))?;
        let mut ctx: Ptr = std::ptr::null_mut();
        ok(retain(&mut ctx, device))?;
        ok(set_ctx(ctx))?;
        let mut module: Ptr = std::ptr::null_mut();
        ok(load(&mut module, ptx.as_ptr() as *const c_void))?;
        let name = CString::new(kernel).ok()?;
        let mut function: Ptr = std::ptr::null_mut();
        ok(get_fn(&mut function, module, name.as_ptr()))?;

        let params_bytes: Vec<u8> = params.iter().flat_map(|p| p.to_ne_bytes()).collect();
        let err_bytes = 0i32.to_ne_bytes();
        let mut inputs: Vec<(*const c_void, usize)> = bufs
            .iter()
            .map(|b| (b.ptr as *const c_void, b.size as usize))
            .collect();
        inputs.push((params_bytes.as_ptr() as *const c_void, params_bytes.len()));
        inputs.push((err_bytes.as_ptr() as *const c_void, err_bytes.len()));

        let mut device_ptrs: Vec<u64> = Vec::new();
        for (host, size) in &inputs {
            let mut dptr = 0u64;
            ok(alloc(&mut dptr, *size))?;
            ok(to_dev(dptr, *host, *size))?;
            device_ptrs.push(dptr);
        }
        let mut args: Vec<Ptr> = device_ptrs
            .iter_mut()
            .map(|p| p as *mut u64 as Ptr)
            .collect();
        let block = 256u32;
        let grid = (params[2].max(1) as u32).div_ceil(block);
        ok(launch(
            function,
            grid,
            1,
            1,
            block,
            1,
            1,
            0,
            std::ptr::null_mut(),
            args.as_mut_ptr(),
            std::ptr::null_mut(),
        ))?;
        ok(sync())?;

        let mut error = 0i32;
        ok(to_host(
            &mut error as *mut i32 as Ptr,
            *device_ptrs.last()?,
            4,
        ))?;
        let mut status = if error != 0 {
            KERNEL_ASSERTION_FAILED
        } else {
            OK
        };
        if status == OK {
            for (buf, dptr) in bufs.iter().zip(&device_ptrs) {
                if buf.written != 0 {
                    ok(to_host(buf.ptr, *dptr, buf.size as usize))?;
                }
            }
        }
        for dptr in &device_ptrs {
            if free(*dptr) != 0 {
                status = UNAVAILABLE;
            }
        }
        Some(status)
    }
}
