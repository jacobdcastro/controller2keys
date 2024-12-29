# controller2keys

A basic rust app that accepts a bluetooth-connected Xbox controller's inputs and maps them to keyboard+mouse events.

I wrote this because I (regretfully) play Minecraft Java Edition on my MacBook. Java Edition does not support controllers, so I wrote this.

## Usage

Literally just connect an Xbox controller to your machine via Bluetooth, and `cargo run`. Check out the terminal to see the inputs.

Keymaps are hard-coded at the moment for testing, but I may add a frontend for more customizable and dynamic mappings down the road.
