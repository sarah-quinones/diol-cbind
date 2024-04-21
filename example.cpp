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
