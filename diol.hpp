#ifndef LIBDIOL_API_HEADER_GUARD
#define LIBDIOL_API_HEADER_GUARD

#include "diol.h"
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <ostream>
#include <span>
#include <sstream>
#include <string>
#include <tuple>
#include <utility>
#include <vector>

namespace diol {

namespace detail {
template <typename F, typename... Ts, std::size_t... Is>
void for_each_impl(std::index_sequence<Is...>, F &&f,
                   std::tuple<Ts...> const &tup) {
  int _unused[] = {0, (f(std::get<Is>(tup)), 0)...};
}
} // namespace detail

template <typename... Ts> struct Tuple {
  std::tuple<Ts...> inner;

  Tuple(std::tuple<Ts...> inner) : inner{std::move(inner)} {}

  friend std::ostream &operator<<(std::ostream &os, Tuple const &tup) {
    os << "[";
    char const *sep = "";
    detail::for_each_impl(
        std::make_index_sequence<sizeof...(Ts)>{},
        [&](auto const &elem) {
          os << sep << elem;
          sep = ", ";
        },
        tup.inner);
    os << "]";
    return os;
  }

  template <std::size_t I> friend decltype(auto) get(Tuple const &t) {
    return std::get<I>(t.inner);
  }

  template <std::size_t I> friend decltype(auto) get(Tuple &t) {
    return std::get<I>(t.inner);
  }

  template <std::size_t I> friend decltype(auto) get(Tuple const &&t) {
    return std::get<I>(std::move(t.inner));
  }

  template <std::size_t I> friend decltype(auto) get(Tuple &&t) {
    return std::get<I>(std::move(t.inner));
  }
};

using Monotonicity = LibDiolMonotonicity;

template <typename T> using Ptr = T *;
template <typename... Ts> using FnPtr = void (*)(Ts...);
struct PlotArg {
  constexpr PlotArg(std::uintptr_t n) noexcept : n{n} {}

  std::uintptr_t n;

  friend std::ostream &operator<<(std::ostream &os, PlotArg arg) {
    os << arg.n;
    return os;
  }
};

struct Bencher {
  Bencher() = delete;

  Bencher(Bencher const &) = delete;
  Bencher &operator=(Bencher const &) = delete;

  Bencher(Bencher &&) = delete;
  Bencher &operator=(Bencher &&) = delete;

  template <typename F> void bench(F f) && {
    libdiol_bencher_bench(
        this->ptr, [](void *data) { std::move (*static_cast<F *>(data))(); },
        static_cast<void *>(std::addressof(f)));
  }

private:
  explicit Bencher(LibDiolBencher *ptr) noexcept : ptr{ptr} {}

  friend struct Bench;
  LibDiolBencher *ptr;
};

template <typename T> struct Function {
  std::string name;
  FnPtr<Bencher, T> f;
};

struct BenchConfig {
  ~BenchConfig() noexcept {
    if (this->ptr != nullptr) {
      libdiol_config_drop(this->ptr);
    }
  }

  BenchConfig() noexcept : ptr{nullptr} {}

  BenchConfig(BenchConfig const &bench) = delete;
  BenchConfig &operator=(BenchConfig const &bench) = delete;

  BenchConfig(BenchConfig &&bench) noexcept : ptr{bench.ptr} {
    bench.ptr = nullptr;
  }
  BenchConfig &operator=(BenchConfig &&bench) noexcept {
    if (&bench != this) {
      auto __tmp = std::move(*this);
      this->ptr = bench.ptr;
      bench.ptr = nullptr;
    }
    return *this;
  }

  static BenchConfig from_args() {
    auto config = libdiol_config_from_args();
    return BenchConfig{config};
  }

  void set_metric(std::string name, Monotonicity mono,
                  double (*metric)(std::uintptr_t n, double time_secs)) {
    libdiol_config_set_metric(this->ptr,
                              LibDiolStringUtf8Ref{
                                  name.data(),
                                  name.size(),
                              },
                              mono, metric);
  }

private:
  explicit BenchConfig(LibDiolConfig *ptr) : ptr{ptr} {}

  friend struct Bench;
  LibDiolConfig *ptr;
};

struct Bench {
  ~Bench() noexcept {
    if (this->ptr != nullptr) {
      libdiol_bench_drop(this->ptr);
    }
  }

  Bench() noexcept : ptr{nullptr} {}

  Bench(Bench const &bench) = delete;
  Bench &operator=(Bench const &bench) = delete;

  Bench(Bench &&bench) noexcept : ptr{bench.ptr} { bench.ptr = nullptr; }
  Bench &operator=(Bench &&bench) noexcept {
    if (&bench != this) {
      auto __tmp = std::move(*this);
      this->ptr = bench.ptr;
      bench.ptr = nullptr;
    }
    return *this;
  }

  static Bench from_config(BenchConfig const &config) {
    auto bench = libdiol_bench_from_config(config.ptr);
    return Bench{bench};
  }

  template <typename T>
  void register_funcs(std::span<Function<T>> funcs, std::span<T> args) {
    std::vector<LibDiolStringUtf8Ref> func_names;
    std::vector<void const *> func_data;
    std::vector<FnPtr<void const *, LibDiolBencher *, void *>> func_ptrs;
    std::vector<void const *> arg_ptrs;

    func_names.reserve(funcs.size());
    func_data.reserve(funcs.size());
    func_ptrs.reserve(funcs.size());

    arg_ptrs.reserve(args.size());

    for (std::size_t i = 0; i < funcs.size(); ++i) {
      func_names.push_back(LibDiolStringUtf8Ref{
          funcs[i].name.data(),
          funcs[i].name.size(),
      });
      func_data.push_back(reinterpret_cast<void const *>(funcs[i].f));
      func_ptrs.push_back([](void const *func_data, LibDiolBencher *bencher,
                             void *arg) {
        auto ptr =
            reinterpret_cast<FnPtr<Bencher, T>>(const_cast<void *>(func_data));
        (*ptr)(Bencher{bencher}, static_cast<T &&>(*static_cast<T *>(arg)));
      });
    }

    for (std::size_t i = 0; i < args.size(); ++i) {
      arg_ptrs.push_back(std::addressof(args[i]));
    }

    libdiol_bench_register(
        this->ptr, func_names.data(), func_data.data(), func_ptrs.data(),
        funcs.size(), arg_ptrs.data(), args.size(),
        +[](void const *ptr) {
          return static_cast<void *>(new T(*Ptr<T const>(ptr)));
        },
        +[](void *ptr) { delete Ptr<T>(ptr); },
        +[](void const *ptr) {
          std::ostringstream s;
          s << (*Ptr<T>(ptr));
          std::string str = std::move(s).str();
          auto data = static_cast<char *>(std::malloc(str.size()));
          std::memcpy(data, str.data(), str.size());
          return LibDiolStringUtf8{
              data, str.size(),
              +[](char *ptr, std::uintptr_t) { std::free(ptr); }};
        },
        std::is_same<T, PlotArg>::value);
  }

  void run() { libdiol_bench_run(this->ptr); }

private:
  explicit Bench(LibDiolBench *ptr) : ptr{ptr} {}

  LibDiolBench *ptr;
};
} // namespace diol

namespace std {
template <typename... Ts>
struct tuple_size<diol::Tuple<Ts...>> : tuple_size<tuple<Ts...>> {};
template <size_t I, typename... Ts>
struct tuple_element<I, diol::Tuple<Ts...>> : tuple_element<I, tuple<Ts...>> {};

} // namespace std

#endif /* LIBDIOL_API_HEADER_GUARD */
