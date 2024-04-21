#ifndef LIBDIOL_HEADER_GUARD
#define LIBDIOL_HEADER_GUARD

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

enum LibDiolMonotonicity
#ifdef __cplusplus
  : uint8_t
#endif // __cplusplus
 {
  None,
  HigherIsBetter,
  LowerIsBetter,
};
#ifndef __cplusplus
typedef uint8_t LibDiolMonotonicity;
#endif // __cplusplus

typedef struct LibDiolBench LibDiolBench;

typedef struct LibDiolBencher LibDiolBencher;

typedef struct LibDiolConfig {

} LibDiolConfig;

typedef struct LibDiolStringUtf8Ref {
  const char *data;
  uintptr_t len;
} LibDiolStringUtf8Ref;

typedef struct LibDiolStringUtf8 {
  char *data;
  uintptr_t len;
  void (*dealloc)(char*, uintptr_t);
} LibDiolStringUtf8;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void libdiol_config_drop(struct LibDiolConfig *config);

void libdiol_bench_drop(struct LibDiolBench *bench);

struct LibDiolConfig *libdiol_config_from_args(void);

struct LibDiolBench *libdiol_bench_from_config(const struct LibDiolConfig *config);

void libdiol_bencher_bench(struct LibDiolBencher *bencher, void (*f)(void*), void *data);

void libdiol_config_set_metric(struct LibDiolConfig *config,
                               struct LibDiolStringUtf8Ref metric_name,
                               LibDiolMonotonicity metric_monotonicity,
                               double (*metric)(uintptr_t, double));

void libdiol_bench_register(struct LibDiolBench *bench,
                            const struct LibDiolStringUtf8Ref *func_names,
                            const void *const *func_data,
                            void (**funcs)(const void *func_data,
                                           struct LibDiolBencher *bencher,
                                           void *arg),
                            uintptr_t n_funcs,
                            const void *const *args,
                            uintptr_t n_args,
                            void *(*arg_clone)(const void*),
                            void (*arg_drop)(void*),
                            struct LibDiolStringUtf8 (*arg_print)(const void*),
                            bool is_plot_arg);

void libdiol_bench_run(struct LibDiolBench *bench);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* LIBDIOL_HEADER_GUARD */
