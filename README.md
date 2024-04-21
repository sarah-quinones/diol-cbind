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
