# otopoiesis

A constructive audio programming environment with timeline-based view.

(*Currently, very early stage of development.*)

[![Netlify Status](https://api.netlify.com/api/v1/badges/45c6bb83-4416-4a20-8364-036931f956a8/deploy-status)](https://app.netlify.com/sites/jovial-starship-05306a/deploys)

Web version demo: https://otopoiesis.matsuuratomoya.com

## Concept

The goal of this project is to create a music creation software with a timeline-based GUI, but one whose project files are more structurally abstract, and can be described and manipulated programmatically.

Simply: Makeing the project file of the DAW software into a source code of program.

- DAW softwares are generally not programmable at all, with a few exception like Max for Live in Ableton Live, Reascript in Reaper,and Lua Scripting in Ardour. However these features are for either just an automation of the software or custom audio-effect or instrument.
- Likewise, many sound programming environments are unit-generator based like a modular synthesizer and do not have timeline-based view. [Blue](https://blue.kunstmusik.com/), a frontend for Csound is also a few exception.
- There are some more general timeline-based programmable sequencing environment like [OSSIA score](https://ossia.io/). otopoisesis is more focusing on linear timeline and less focusing on real-time interaction between external events.
  


## How to build

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


