# `msh-rw`
Read and write Gmsh `msh` files

⚠️ This library is in an alpha state ⚠️

The minimum supported `rustc` version is `1.40.0`.

# Supported versions
* `msh` 2.2 ascii (WIP)

# Planned
* `msh` 2.2 (binary)
* `msh` 4.1 (ascii)
* `msh` 4.1 (ascii)

### Ideas
Check out `itoa`, `dtoa` to see if they make a noticable difference. 

### `serde` integration 
Enable `serde` support with

`cargo build --features serde`

Run the `serde` tests with 

`cargo test --features serde`
