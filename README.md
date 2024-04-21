# installation
- copy the `diol.h` and `diol.hpp` headers to your include directory.
- compile the `diol` static lib with `cargo build --release` and link `.target/release/libdiol_cbind.a` to your program.

# example

```cpp
#include "diol.hpp"

using namespace diol;

void foo(Bencher bencher, PlotArg arg) {
  auto v = std::vector<double>(arg.n);

  std::move(bencher).bench([&] {
    for (auto &x : v) {
      x *= 2.0;
    }
  });
}

void bar(Bencher bencher, PlotArg arg) {
  auto v = std::vector<double>(arg.n);

  std::move(bencher).bench([&] {
    for (auto &x : v) {
      x *= 0.1;
    }
  });
}

int main() {
  auto config = BenchConfig::from_args();
  config.set_metric("f64/s", HigherIsBetter,
                    [](std::uintptr_t n, double time) { return n / time; });

  auto bench = Bench::from_config(config);

  Function<PlotArg> funcs[] = {{"foo", foo}, {"bar", bar}};
  PlotArg args[] = {1, 2, 4, 512};

  bench.register_funcs<PlotArg>(funcs, args);
  bench.run();
}
```

# output
```
╭────────────────┬──────┬──────────┬───────────┬───────────┬───────────┬───────────╮
│ benchmark      │ args │    f64/s │   fastest │    median │      mean │    stddev │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │    1 │  4.874e8 │   2.05 ns │   2.05 ns │   2.05 ns │  44.00 ps │
│ bar            │    1 │  4.877e8 │   2.05 ns │   2.05 ns │   2.05 ns │  36.00 ps │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │    2 │  9.719e8 │   2.05 ns │   2.05 ns │   2.06 ns │  94.00 ps │
│ bar            │    2 │  9.693e8 │   2.05 ns │   2.05 ns │   2.06 ns │  67.00 ps │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │    4 │  1.459e9 │   2.73 ns │   2.73 ns │   2.75 ns │ 112.00 ps │
│ bar            │    4 │  1.349e9 │   2.73 ns │   2.96 ns │   2.96 ns │  62.00 ps │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │  512 │ 1.755e10 │  29.05 ns │  29.12 ns │  29.19 ns │ 781.00 ps │
│ bar            │  512 │ 1.757e10 │  29.07 ns │  29.10 ns │  29.15 ns │ 452.00 ps │
╰────────────────┴──────┴──────────┴───────────┴───────────┴───────────┴───────────╯
```
