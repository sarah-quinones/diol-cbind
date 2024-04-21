use diol_cbind::*;
use std::{ffi::*, ptr::null};

fn main() {
    unsafe {
        let config = libdiol_config_from_args();
        let bench = libdiol_bench_from_config(config);
        libdiol_config_drop(config);

        unsafe extern "C" fn foo(_: *const c_void, bencher: *mut LibDiolBencher, arg: *mut c_void) {
            unsafe extern "C" fn f(data: *mut c_void) {
                let data = &mut *(data as *mut Vec<f64>);
                for x in data {
                    *x *= 2.0;
                }
            }

            let arg = *(arg as *const u32);
            let mut data = vec![0.0; arg as usize];
            let data = (&mut data) as *mut _ as *mut c_void;
            libdiol_bencher_bench(bencher, f, data);
        }
        unsafe extern "C" fn bar(_: *const c_void, bencher: *mut LibDiolBencher, arg: *mut c_void) {
            unsafe extern "C" fn f(data: *mut c_void) {
                let data = &mut *(data as *mut Vec<f64>);
                for x in data {
                    *x *= 0.1;
                }
            }

            let arg = *(arg as *const u32);
            let mut data = vec![0.0; arg as usize];
            let data = (&mut data) as *mut _ as *mut c_void;
            libdiol_bencher_bench(bencher, f, data);
        }

        let names = ["foo", "bar"];
        libdiol_bench_register(
            bench,
            names
                .map(|str| LibDiolStringUtf8Ref {
                    data: str.as_ptr() as *const i8,
                    len: str.len(),
                })
                .as_ptr(),
            [null(), null()].as_ptr(),
            [foo, bar].as_ptr(),
            2,
            [
                (&1u32) as *const u32 as *const c_void,
                (&3u32) as *const u32 as *const c_void,
                (&9u32) as *const u32 as *const c_void,
            ]
            .as_ptr(),
            3,
            {
                unsafe extern "C" fn f(x: *const c_void) -> *mut c_void {
                    Box::into_raw(Box::new(*(x as *const u32))) as *mut c_void
                }
                f
            },
            {
                unsafe extern "C" fn f(_: *mut c_void) {}
                f
            },
            {
                unsafe extern "C" fn f(x: *const c_void) -> LibDiolStringUtf8 {
                    let x = *(x as *const u32);
                    let str = Box::into_raw(format!("{x}").into_bytes().into_boxed_slice());
                    LibDiolStringUtf8 {
                        data: str as *mut i8,
                        len: (*str).len(),
                        dealloc: {
                            unsafe extern "C" fn f(ptr: *mut c_char, len: usize) {
                                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                                    ptr as *mut u8,
                                    len,
                                )));
                            }
                            f
                        },
                    }
                }
                f
            },
            false,
        );

        libdiol_bench_run(bench);
        libdiol_bench_drop(bench);
    }
}
