## 0.3.0

- Build script (and erst as build-dep) is no longer necessary. Still need to use `erst-prepare` if using dynamic (e.g. 
`erst-prepare && cargo run`).
- Building in release mode negates `dynamic`. No need to change features flags to build release.
