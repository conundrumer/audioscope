Audioscope
==========

A collection of audio visualizers true to the sound.

Implemented views:

- Analytic (Hilbert Scope)

Running
-------

At the project root:

`cargo run --release -- config.toml`

Audioscope will use your default audio input device, which will most likely be a microphone. If you fiddle with your audio settings enough (e.g. Soundflower on OSX), you can get anything into Audioscope.

You can move the binary elsewhere, but currently, it dynamically loads the shaders, so `src/glsl` needs to be on the current path.

Additional Documentation
-------------

[A high level explanation](https://medium.com/@conundrumer/a-perceptually-meaningful-audio-visualizer-ee72051781bc#.p87d5rrxg)
[Project Trello](https://trello.com/b/je2p03G7/audioscope)
