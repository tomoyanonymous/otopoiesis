# otopoiesis

a constructive audio programming environment with timeline-based view.


# How to build

### native
```sh
cargo run
```

### web

[![Netlify Status](https://api.netlify.com/api/v1/badges/45c6bb83-4416-4a20-8364-036931f956a8/deploy-status)](https://app.netlify.com/sites/jovial-starship-05306a/deploys)

https://otopoiesis.matsuuratomoya.com

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


