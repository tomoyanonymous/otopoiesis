# otopoiesis

a constructive audio programming environment with timeline-based view.


# How to build

### native
```sh
cargo run
```

### web

```sh
cargo build --target wasm32-unknown-unknown --features "web"
```

Debug with wasm-pack.

```sh
npm install
npm start
```

# Todo

- [ ] project file export/import
- [ ] Wav file loading
  - [ ] file io streaming(wasm compatibility??) / caching system
  - [ ] thumbnail generation for audio files
  - [ ] drug & drop of file?
- [ ] Channel adaptation between different configurations of channels
- [ ] Audio region/track transformer
  - [ ] region reprecator
  - [x] fadein/out filter
- [ ] primitive scripting environment

(c) Tomoya Matsuura/松浦知也 2022


