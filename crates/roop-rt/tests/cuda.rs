use roop_rt::roop_gpu_dispatch;

#[repr(C)]
struct Buffer {
    ptr: *mut std::ffi::c_void,
    size: i64,
    written: i32,
}

const PTX: &[u8] = br#".version 8.0
.target sm_80
.address_size 64

.visible .entry increment(
    .param .u64 values,
    .param .u64 params,
    .param .u64 err
)
{
    .reg .pred %p;
    .reg .b32 %r;
    .reg .b64 %rd<8>;

    ld.param.u64 %rd1, [values];
    ld.param.u64 %rd2, [params];
    ld.global.u64 %rd3, [%rd2+16];
    mov.u32 %r, %tid.x;
    cvt.u64.u32 %rd4, %r;
    setp.ge.s64 %p, %rd4, %rd3;
    @%p bra done;
    mul.wide.u32 %rd5, %r, 8;
    add.u64 %rd6, %rd1, %rd5;
    ld.global.s64 %rd7, [%rd6];
    add.s64 %rd7, %rd7, 1;
    st.global.s64 [%rd6], %rd7;
done:
    ret;
}
"#;

#[test]
fn cuda_driver_launches_a_ptx_kernel() {
    let mut values = [0_i64, 1, 2, 3];
    let buf = Buffer {
        ptr: values.as_mut_ptr().cast(),
        size: std::mem::size_of_val(&values) as i64,
        written: 1,
    };

    let kernel = c"increment";
    let ptx = [PTX, b"\0"].concat();
    let status = unsafe {
        roop_gpu_dispatch(
            1,
            ptx.as_ptr().cast(),
            ptx.len() as i64,
            kernel.as_ptr(),
            (&buf as *const Buffer).cast(),
            1,
            0,
            1,
            values.len() as i64,
        )
    };
    if status == 2 {
        eprintln!("skipping CUDA test: driver unavailable");
        return;
    }
    assert_eq!(status, 0);
    assert_eq!(values, [1, 2, 3, 4]);
}
