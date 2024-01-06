# M8-kbd

USB keypad designed for use with the [M8 Headless](https://github.com/Dirtywave/M8HeadlessFirmware). The layout is inspired by the original [Dirtywave M8 tracker](https://dirtywave.com/).

Not a replacement for the original hardware, but something I put together with spare parts as a way to get hands-on experience with the M8.

Features:

- 3x3 handwired keyboard matrix (1N4148 diodes)
- Kailh Choc V1 switches ([datasheet](https://cdn-shop.adafruit.com/product-files/5113/CHOC+keyswitch_Kailh-CPG135001D01_C400229.pdf)), mounted in 13.85x13.85 square holes
- [Raspberry Pico](https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf) as the keyboard controller
- Firmware in Rust, based on [rp2040-hal](https://github.com/rp-rs/rp-hal-boards/tree/main/boards/rp-pico), [RTIC v1](https://rtic.rs/1/book/en/) and [Keyberon](https://github.com/TeXitoi/keyberon)
- Keymap contains arrow keys, shift, space, 'z'  and 'x' ([m8.run](https://m8.run) defaults)
- Programmed and debugged over SWD with [probe-rs](https://probe.rs/) and [Raspberry Pi Debug Probe](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html)
- Bottom plate also has a mount for [Teensy 4.1](https://www.pjrc.com/store/teensy41.html) (with headers) which runs M8 Headless.
 
![top plate](img/top-plate.jpeg)
![perspective view](img/perspective.jpeg)
![side view](img/side.jpeg)
![bottom plate](img/bottom-plate.jpeg)
![matrix](img/wiring.jpeg)
