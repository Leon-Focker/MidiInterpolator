# MidiInterpolator

MidiInterpolator is a MIDI event processing plugin that can interpolate between Notes of two Midi Channels.

## Building

Precompiled binaries can be found in the [Releases tab](https://github.com/Leon-Focker/MidiInterpolator/releases/)

On macOS you may need to [disable Gatekeeper](https://disable-gatekeeper.github.io/) to be able to use this plugin.

After installing [Rust](https://rustup.rs/), you can compile MidiInterpolator yourself with this command:

```shell
cargo xtask bundle midiinterpolator --release
```
