# installation
- copy the `diol.h` and `diol.hpp` headers to your include directory.
- compile the `diol` static lib with `cargo build --release` and link `.target/release/libdiol_cbind.a` to your program.

# example

```cpp
#include "diol.h"
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

void foo_tup(Bencher bencher, Tuple<int, int> arg) {
  auto [m, n] = arg;
  auto v = std::vector<double>(n);
  std::move(bencher).bench([&] {
    for (auto &x : v) {
      x *= 2.0;
    }
  });
}

void bar_tup(Bencher bencher, Tuple<int, int> arg) {
  auto [m, n] = arg;
  auto v = std::vector<double>(n);
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

  {
    Function<PlotArg> funcs[] = {{"foo", foo}, {"bar", bar}};
    PlotArg args[] = {1, 2, 4, 512};
    bench.register_funcs<PlotArg>(funcs, args);
  }

  {
    Function<Tuple<int, int>> funcs[] = {{"foo", foo_tup}, {"bar", bar_tup}};
    Tuple<int, int> args[] = {{{1, 2}}, {{2, 4}}, {{4, 6}}, {{512, 768}}};
    bench.register_funcs<Tuple<int, int>>(funcs, args);
  }
  bench.run();
}
```

# output
```
╭────────────────┬──────┬──────────┬───────────┬───────────┬───────────┬───────────╮
│ benchmark      │ args │    f64/s │   fastest │    median │      mean │    stddev │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │    1 │  4.869e8 │   2.05 ns │   2.05 ns │   2.06 ns │ 141.00 ps │
│ bar            │    1 │  4.868e8 │   2.05 ns │   2.05 ns │   2.06 ns │ 107.00 ps │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │    2 │  9.743e8 │   2.05 ns │   2.05 ns │   2.06 ns │ 102.00 ps │
│ bar            │    2 │  9.741e8 │   2.05 ns │   2.05 ns │   2.06 ns │ 100.00 ps │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │    4 │  1.455e9 │   2.73 ns │   2.73 ns │   2.76 ns │ 266.00 ps │
│ bar            │    4 │  1.349e9 │   2.73 ns │   2.96 ns │   2.97 ns │ 248.00 ps │
├────────────────┼──────┼──────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │  512 │ 1.038e10 │  49.12 ns │  49.13 ns │  49.41 ns │   3.10 ns │
│ bar            │  512 │ 1.039e10 │  49.12 ns │  49.13 ns │  49.37 ns │   2.87 ns │
╰────────────────┴──────┴──────────┴───────────┴───────────┴───────────┴───────────╯
╭────────────────┬────────────┬───────────┬───────────┬───────────┬───────────╮
│ benchmark      │       args │   fastest │    median │      mean │    stddev │
├────────────────┼────────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │     [1, 2] │   2.05 ns │   2.05 ns │   2.06 ns │ 154.00 ps │
│ bar            │     [1, 2] │   2.05 ns │   2.05 ns │   2.06 ns │  94.00 ps │
├────────────────┼────────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │     [2, 4] │   2.95 ns │   2.96 ns │   2.97 ns │ 171.00 ps │
│ bar            │     [2, 4] │   2.96 ns │   3.41 ns │   3.30 ns │ 270.00 ps │
├────────────────┼────────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │     [4, 6] │   2.96 ns │   2.96 ns │   2.99 ns │ 246.00 ps │
│ bar            │     [4, 6] │   3.41 ns │   3.41 ns │   3.47 ns │ 281.00 ps │
├────────────────┼────────────┼───────────┼───────────┼───────────┼───────────┤
│ foo            │ [512, 768] │  72.78 ns │  73.23 ns │  76.05 ns │   6.65 ns │
│ bar            │ [512, 768] │  73.66 ns │  81.55 ns │  83.48 ns │   8.28 ns │
╰────────────────┴────────────┴───────────┴───────────┴───────────┴───────────╯
```

# screenshot
visualization can be done using [`estra`](https://github.com/sarah-ek/estra/).

![image](https://github.com/sarah-ek/diol-cbind/assets/40109184/4de9e290-0180-40bc-8857-5cb8a277f33d)

