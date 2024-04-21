use std::any::TypeId;
use std::ffi::c_char;
use std::ffi::c_void;
use std::fmt;

use diol::config::BenchConfig;
use diol::config::PlotMetric;
use diol::traits::Arg;
use diol::traits::Monotonicity;
use diol::traits::Register;
use diol::Bench;
use diol::Bencher;
use diol::PlotArg;

fn nounwind<R>(f: impl FnOnce() -> R) -> R {
    struct PanicDrop;
    impl Drop for PanicDrop {
        fn drop(&mut self) {
            panic!();
        }
    }
    let guard = PanicDrop;
    let r = f();
    std::mem::forget(guard);
    r
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibDiolPlotArg {
    pub n: usize,
}

pub struct LibDiolResult {
    __private: (),
}

pub struct LibDiolBench {
    __private: (),
}

pub struct LibDiolBencher {
    __private: (),
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibDiolStringUtf8Ref {
    pub data: *const c_char,
    pub len: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibDiolStringUtf8 {
    pub data: *mut c_char,
    pub len: usize,
    pub dealloc: unsafe extern "C" fn(*mut c_char, usize),
}

struct StringUtf8 {
    pub data: *mut c_char,
    pub len: usize,
    pub dealloc: unsafe extern "C" fn(*mut c_char, usize),
}

impl Drop for StringUtf8 {
    fn drop(&mut self) {
        unsafe { (self.dealloc)(self.data, self.len) }
    }
}

impl LibDiolStringUtf8Ref {
    unsafe fn to_str(self) -> Option<&'static str> {
        if self.len == usize::MAX {
            None
        } else if self.data.is_null() {
            equator::assert!(self.len == 0);
            Some("")
        } else {
            Some(
                std::str::from_utf8(std::slice::from_raw_parts(self.data as *const u8, self.len))
                    .unwrap(),
            )
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum LibDiolMonotonicity {
    None,
    HigherIsBetter,
    LowerIsBetter,
}

/// colors to use for the generated plot
#[derive(Copy, Clone)]
pub enum LibDiolPlotColors {
    CubehelixDefault,
    Turbo,
    Spectral,
    Viridis,
    Magma,
    Inferno,
    Plasma,
    Cividis,
    Warm,
    Cool,
}

#[repr(C)]
pub struct LibDiolConfig {
    __private: (),
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_config_drop(config: *mut LibDiolConfig) {
    nounwind(|| {
        if !config.is_null() {
            drop(Box::from_raw(config as *mut BenchConfig));
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_bench_drop(bench: *mut LibDiolBench) {
    nounwind(|| {
        if !bench.is_null() {
            drop(Box::from_raw(bench as *mut Bench));
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_config_from_args() -> *mut LibDiolConfig {
    nounwind(|| Box::into_raw(Box::new(BenchConfig::from_args())) as *mut _)
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_bench_from_config(
    config: *const LibDiolConfig,
) -> *mut LibDiolBench {
    nounwind(|| Box::into_raw(Box::new(Bench::new(&*(config as *const BenchConfig)))) as *mut _)
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_bencher_bench(
    bencher: *mut LibDiolBencher,
    f: unsafe extern "C" fn(*mut c_void),
    data: *mut c_void,
) {
    nounwind(|| {
        let bencher = std::ptr::read(bencher as *mut Bencher<'_>);
        bencher.bench(|| f(data));
    })
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_config_set_metric(
    config: *mut LibDiolConfig,
    metric_name: LibDiolStringUtf8Ref,
    metric_monotonicity: LibDiolMonotonicity,
    metric: extern "C" fn(usize, f64) -> f64,
) {
    let config = &mut *(config as *mut BenchConfig);

    #[derive(Clone)]
    struct FfiMetric {
        name: String,
        monotonicity: Monotonicity,
        metric: extern "C" fn(usize, f64) -> f64,
    }

    impl diol::traits::PlotMetric for FfiMetric {
        fn compute(&self, arg: PlotArg, time: diol::Picoseconds) -> f64 {
            (self.metric)(arg.0, time.to_secs())
        }

        fn monotonicity(&self) -> Monotonicity {
            self.monotonicity
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    config.plot_metric = PlotMetric::new(FfiMetric {
        name: metric_name.to_str().unwrap().to_string(),
        monotonicity: match metric_monotonicity {
            LibDiolMonotonicity::None => Monotonicity::None,
            LibDiolMonotonicity::HigherIsBetter => Monotonicity::HigherIsBetter,
            LibDiolMonotonicity::LowerIsBetter => Monotonicity::LowerIsBetter,
        },
        metric,
    });
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_bench_register(
    bench: *mut LibDiolBench,
    func_names: *const LibDiolStringUtf8Ref,
    func_data: *const *const c_void,
    funcs: *const unsafe extern "C" fn(
        func_data: *const c_void,
        bencher: *mut LibDiolBencher,
        arg: *mut c_void,
    ),
    n_funcs: usize,
    args: *const *const c_void,
    n_args: usize,
    arg_clone: unsafe extern "C" fn(*const c_void) -> *mut c_void,
    arg_drop: unsafe extern "C" fn(*mut c_void),
    arg_print: unsafe extern "C" fn(*const c_void) -> LibDiolStringUtf8,
    is_plot_arg: bool,
) {
    nounwind(|| {
        let bench = &mut *(bench as *mut Bench);
        let func_names = std::slice::from_raw_parts(func_names, n_funcs);
        let func_data = std::slice::from_raw_parts(func_data, n_funcs);
        let funcs = std::slice::from_raw_parts(funcs, n_funcs);
        if is_plot_arg {
            let args = std::slice::from_raw_parts(args, n_args);
            bench.register_many_dyn(
                func_names
                    .iter()
                    .map(|s| unsafe { s.to_str().unwrap().to_string() })
                    .collect(),
                funcs
                    .iter()
                    .zip(func_data)
                    .map(|(&f, &data)| {
                        Box::new(move |bencher: diol::Bencher<'_>, arg: Box<dyn Arg>| {
                            let arg = (&*arg) as *const dyn Arg as *const PlotArg;
                            let arg = *arg;
                            f(
                                data,
                                (&mut { bencher }) as *mut diol::Bencher<'_> as *mut LibDiolBencher,
                                (&mut { arg }) as *mut PlotArg as *mut c_void,
                            );
                        }) as Box<dyn Register<Box<dyn Arg>>>
                    })
                    .collect(),
                TypeId::of::<PlotArg>(),
                args.iter()
                    .copied()
                    .map(|arg| Box::new(*(arg as *const PlotArg)) as Box<dyn Arg>)
                    .collect(),
            );
        } else {
            struct DynArg {
                data: *mut c_void,
                clone: unsafe extern "C" fn(*const c_void) -> *mut c_void,
                drop: unsafe extern "C" fn(*mut c_void),
                print: unsafe extern "C" fn(*const c_void) -> LibDiolStringUtf8,
            }

            impl Clone for DynArg {
                fn clone(&self) -> Self {
                    Self {
                        data: unsafe { (self.clone)(self.data) },
                        clone: self.clone,
                        drop: self.drop,
                        print: self.print,
                    }
                }
            }
            impl Drop for DynArg {
                fn drop(&mut self) {
                    unsafe { (self.drop)(self.data) }
                }
            }

            impl fmt::Debug for DynArg {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let LibDiolStringUtf8 { data, len, dealloc } =
                        unsafe { (self.print)(self.data) };
                    let str = StringUtf8 { data, len, dealloc };
                    let str = LibDiolStringUtf8Ref {
                        data: str.data,
                        len: str.len,
                    };
                    f.write_str(unsafe { str.to_str().unwrap() })
                }
            }

            let args = std::slice::from_raw_parts(args, n_args);
            bench.register_many_dyn(
                func_names
                    .iter()
                    .map(|s| unsafe { s.to_str().unwrap().to_string() })
                    .collect(),
                funcs
                    .iter()
                    .zip(func_data)
                    .map(|(&f, &data)| {
                        Box::new(move |bencher: diol::Bencher<'_>, arg: Box<dyn Arg>| {
                            let arg = (&*arg) as *const dyn Arg as *const DynArg;
                            let arg = (*arg).data;
                            f(
                                data,
                                (&mut { bencher }) as *mut diol::Bencher<'_> as *mut LibDiolBencher,
                                arg,
                            );
                        }) as Box<dyn Register<Box<dyn Arg>>>
                    })
                    .collect(),
                TypeId::of::<DynArg>(),
                args.iter()
                    .copied()
                    .map(|arg| DynArg {
                        data: arg_clone(arg),
                        clone: arg_clone,
                        drop: arg_drop,
                        print: arg_print,
                    })
                    .map(|arg| Box::new(arg) as Box<dyn Arg>)
                    .collect(),
            );
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn libdiol_bench_run(bench: *mut LibDiolBench) {
    nounwind(|| {
        let bench = &mut *(bench as *mut Bench);
        bench.run().unwrap();
    })
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use super::*;

    #[test]
    fn test_bindings() {
        unsafe {
            let config = libdiol_config_from_args();
            let bench = libdiol_bench_from_config(config);
            libdiol_config_drop(config);

            unsafe extern "C" fn foo(
                _: *const c_void,
                bencher: *mut LibDiolBencher,
                arg: *mut c_void,
            ) {
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
            unsafe extern "C" fn bar(
                _: *const c_void,
                bencher: *mut LibDiolBencher,
                arg: *mut c_void,
            ) {
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

            libdiol_bench_drop(bench);
        }
    }
}
