# Voxel
A voxel renderer I've written mainly to learn how graphics programming works.

## Features
- Some basic optimizations (frustum culling, tightly packed small vertices)
- Basic fog implementation
- Multithreaded chunk generation and meshing using [`rayon`](https://github.com/rayon-rs/rayon)

## Screenshots
![Screenshot 1](./screenshots/screenshot1.png)
![Screenshot 2](./screenshots/screenshot2.png)
![Screenshot 3](./screenshots/screenshot3.png)

## Running
Build and run:

```sh
cargo run --release
```

or download binaries from GitHub releases.

In the future, I might implement support for running it in the browser using WASM, though multithreading in WASM requires extra setup, so it may take a while to implement properly.
