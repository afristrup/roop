use crate::gpu::{KERNEL_ASSERTION_FAILED, OK, RoopBuf, UNAVAILABLE};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CString, c_char, c_void};
use std::sync::{Mutex, OnceLock};

type Status = i32;
type Ptr = *mut c_void;

struct Module {
    handle: usize,
    functions: HashMap<String, usize>,
}

struct Cuda {
    _library: Library,
    set_context: unsafe extern "C" fn(Ptr) -> Status,
    load_module: unsafe extern "C" fn(*mut Ptr, *const c_void) -> Status,
    get_function: unsafe extern "C" fn(*mut Ptr, Ptr, *const c_char) -> Status,
    allocate: unsafe extern "C" fn(*mut u64, usize) -> Status,
    copy_to_device: unsafe extern "C" fn(u64, *const c_void, usize) -> Status,
    copy_to_host: unsafe extern "C" fn(*mut c_void, u64, usize) -> Status,
    free: unsafe extern "C" fn(u64) -> Status,
    launch: unsafe extern "C" fn(
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
    ) -> Status,
    synchronize: unsafe extern "C" fn() -> Status,
    context: usize,
    modules: Mutex<HashMap<Vec<u8>, Module>>,
}

macro_rules! driver {
    ($lib:expr, $name:literal, $ty:ty) => {{
        let symbol: Symbol<$ty> = $lib.get(concat!($name, "\0").as_bytes()).ok()?;
        *symbol
    }};
}

impl Cuda {
    unsafe fn open() -> Option<Self> {
        let library = ["libcuda.so.1", "libcuda.so", "nvcuda.dll"]
            .into_iter()
            .find_map(|name| unsafe { Library::new(name).ok() })?;
        unsafe {
            let init = driver!(library, "cuInit", unsafe extern "C" fn(u32) -> Status);
            let device_get = driver!(
                library,
                "cuDeviceGet",
                unsafe extern "C" fn(*mut i32, i32) -> Status
            );
            let retain_context = driver!(
                library,
                "cuDevicePrimaryCtxRetain",
                unsafe extern "C" fn(*mut Ptr, i32) -> Status
            );
            let set_context = driver!(
                library,
                "cuCtxSetCurrent",
                unsafe extern "C" fn(Ptr) -> Status
            );
            let load_module = driver!(
                library,
                "cuModuleLoadData",
                unsafe extern "C" fn(*mut Ptr, *const c_void) -> Status
            );
            let get_function = driver!(
                library,
                "cuModuleGetFunction",
                unsafe extern "C" fn(*mut Ptr, Ptr, *const c_char) -> Status
            );
            let allocate = driver!(
                library,
                "cuMemAlloc_v2",
                unsafe extern "C" fn(*mut u64, usize) -> Status
            );
            let copy_to_device = driver!(
                library,
                "cuMemcpyHtoD_v2",
                unsafe extern "C" fn(u64, *const c_void, usize) -> Status
            );
            let copy_to_host = driver!(
                library,
                "cuMemcpyDtoH_v2",
                unsafe extern "C" fn(*mut c_void, u64, usize) -> Status
            );
            let free = driver!(library, "cuMemFree_v2", unsafe extern "C" fn(u64) -> Status);
            let launch = driver!(
                library,
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
            let synchronize = driver!(
                library,
                "cuCtxSynchronize",
                unsafe extern "C" fn() -> Status
            );

            ok(init(0))?;
            let mut device = 0;
            ok(device_get(&mut device, 0))?;
            let mut context = std::ptr::null_mut();
            ok(retain_context(&mut context, device))?;
            ok(set_context(context))?;
            Some(Self {
                _library: library,
                set_context,
                load_module,
                get_function,
                allocate,
                copy_to_device,
                copy_to_host,
                free,
                launch,
                synchronize,
                context: context as usize,
                modules: Mutex::new(HashMap::new()),
            })
        }
    }

    fn function(&self, ptx: &[u8], kernel: &str) -> Option<Ptr> {
        unsafe {
            ok((self.set_context)(self.context as Ptr))?;
            let mut modules = self.modules.lock().ok()?;
            let module = match modules.entry(ptx.to_vec()) {
                std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let mut handle = std::ptr::null_mut();
                    ok((self.load_module)(
                        &mut handle,
                        ptx.as_ptr() as *const c_void,
                    ))?;
                    entry.insert(Module {
                        handle: handle as usize,
                        functions: HashMap::new(),
                    })
                }
            };
            match module.functions.entry(kernel.into()) {
                std::collections::hash_map::Entry::Occupied(entry) => Some(*entry.get() as Ptr),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let name = CString::new(kernel).ok()?;
                    let mut function = std::ptr::null_mut();
                    ok((self.get_function)(
                        &mut function,
                        module.handle as Ptr,
                        name.as_ptr(),
                    ))?;
                    entry.insert(function as usize);
                    Some(function)
                }
            }
        }
    }
}

fn cuda() -> Option<&'static Cuda> {
    static CUDA: OnceLock<Option<Cuda>> = OnceLock::new();
    CUDA.get_or_init(|| unsafe { Cuda::open() }).as_ref()
}

fn ok(status: Status) -> Option<()> {
    (status == 0).then_some(())
}

/// Runs a PTX kernel through the CUDA driver API. Modules and the primary
/// context stay cached while the process runs; each call still copies its buffers.
pub fn cuda_dispatch(ptx: &[u8], kernel: &str, bufs: &[RoopBuf], params: [i64; 3]) -> i32 {
    let Some(cuda) = cuda() else {
        return UNAVAILABLE;
    };
    let Some(function) = cuda.function(ptx, kernel) else {
        return UNAVAILABLE;
    };
    unsafe { launch(cuda, function, bufs, params) }.unwrap_or(UNAVAILABLE)
}

unsafe fn launch(cuda: &Cuda, function: Ptr, bufs: &[RoopBuf], params: [i64; 3]) -> Option<i32> {
    unsafe {
        let params_bytes: Vec<u8> = params.iter().flat_map(|p| p.to_ne_bytes()).collect();
        let err_bytes = 0i32.to_ne_bytes();
        let mut inputs: Vec<(*const c_void, usize)> = bufs
            .iter()
            .map(|b| (b.ptr as *const c_void, b.size as usize))
            .collect();
        inputs.push((params_bytes.as_ptr() as *const c_void, params_bytes.len()));
        inputs.push((err_bytes.as_ptr() as *const c_void, err_bytes.len()));

        let mut device_ptrs = Vec::new();
        for (host, size) in &inputs {
            let mut dptr = 0u64;
            ok((cuda.allocate)(&mut dptr, *size))?;
            ok((cuda.copy_to_device)(dptr, *host, *size))?;
            device_ptrs.push(dptr);
        }
        let mut args: Vec<Ptr> = device_ptrs
            .iter_mut()
            .map(|p| p as *mut u64 as Ptr)
            .collect();
        let block = 256u32;
        let grid = (params[2].max(1) as u32).div_ceil(block);
        ok((cuda.launch)(
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
        ok((cuda.synchronize)())?;

        let mut error = 0i32;
        ok((cuda.copy_to_host)(
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
                    ok((cuda.copy_to_host)(buf.ptr, *dptr, buf.size as usize))?;
                }
            }
        }
        for dptr in &device_ptrs {
            if (cuda.free)(*dptr) != 0 {
                status = UNAVAILABLE;
            }
        }
        Some(status)
    }
}
