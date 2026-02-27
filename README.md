# m5-sys

Low-level Rust bindings to the m5 utility library of the
[gem5](https://www.gem5.org/) architectural simulator.

## Native Dependency

There are a few ways for this crate to link against m5.

- If environment variable `GEM5_M5_LIB_DIR` is set, this crate will look for
  a compiled m5 library under said path.
  - If environment variable `GEM5_M5_INCLUDE` is set, this crate will add it
    to the include search path of bindgen.
  - Otherwise, this crate will guess the include path is
    `${GEM5_M5_LIB_DIR}/../include`.
- Else, if `GEM5_M5_BUILD` is unset,
  this crate will make no effort to search for m5 other than the
  system default of `/usr/lib` and `/usr/include`.
- Else, if `GEM5_SRC` is set,
  this crate will try to build m5 using the source at said path and use this
  directory for the include path of bindgen.
- Else, if `GEM5_SRC` is unset,
  this crate will download a release tarball of gem5 source code and attempt
  to build m5 from it.
