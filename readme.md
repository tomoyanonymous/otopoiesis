# otopoiesis

[![.github/workflows/build_native.yml](https://github.com/tomoyanonymous/otopoiesis/actions/workflows/build_native.yml/badge.svg)](https://github.com/tomoyanonymous/otopoiesis/actions/workflows/build_native.yml) [![.github/workflows/build_web.yml](https://github.com/tomoyanonymous/otopoiesis/actions/workflows/build_web.yml/badge.svg)](https://github.com/tomoyanonymous/otopoiesis/actions/workflows/build_web.yml) [![Netlify Status](https://api.netlify.com/api/v1/badges/45c6bb83-4416-4a20-8364-036931f956a8/deploy-status)](https://app.netlify.com/sites/jovial-starship-05306a/deploys)

A constructive audio programming environment with timeline-based view.

(*Currently, very early stage of development.*)


Web version demo: **https://otopoiesis.matsuuratomoya.com**

## Concept

The goal of this project is to create a music creation software with a timeline-based GUI, but one whose project files are more structurally abstract, and can be described and manipulated programmatically.

Simply: Makeing the project file of the DAW software into a source code of program.

- DAW softwares are generally not programmable at all, with a few exception like Max for Live in Ableton Live, Reascript in Reaper,and Lua Scripting in Ardour. However these features are for either just an automation of the software or custom audio-effect or instrument.
- Likewise, many sound programming environments are unit-generator based like a modular synthesizer and do not have timeline-based view. [Blue](https://blue.kunstmusik.com/), a frontend for Csound is also a few exception.
- There are some more general timeline-based programmable sequencing environment like [OSSIA score](https://ossia.io/). otopoisesis is more focusing on linear timeline and less focusing on real-time interaction between external events.
  


## How to build by yourself

### native

On linux, you need to install ALSA.

```sh
sudo apt-get install libasound2-dev
```

```sh
cargo run
```

### web

Can build & debug with wasm-pack.

```sh
cargo install wasm-pack
npm install
```

#### build

```sh
npm run build 
```

### debug

```sh
npm start
```


# Todo

- [ ] project file export/import
- [ ] Wav file loading
  - [ ] file io streaming / caching system
    - [x] open wav file in native app
  - [ ] thumbnail generation for audio files
  - [ ] drug & drop of file?
- [ ] Channel adaptation between different configurations of channels
- [ ] Audio region/track transformer
  - [ ] region reprecator
  - [x] fadein/out filter
- [ ] primitive scripting environment

(c) Tomoya Matsuura/松浦知也 2022


