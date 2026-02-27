# m5-sys

Low-level Rust bindings to the m5 utility library of the
[gem5](https://www.gem5.org/) architectural simulator.

## Finding gem5 source

There are two ways for this crate to link against m5.

- By default, this crate will download the official gem5 release
  from GitHub and build the m5 library.
- Set the `GEM5_SRC` environment variable to the root of the gem5
  source tree to build from that instead.
