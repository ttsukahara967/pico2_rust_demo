# pico2_rust_demo

Bare-metal Rust on a real **Raspberry Pi Pico 2 W** (RP2350, Cortex-M33), built and flashed from a Mac.
The firmware prints `Hello from Pico 2 W #n` over USB serial once a second. No wiring needed.

Built on [Embassy](https://embassy.dev/) (`embassy-rp` with the `rp235xa` feature). The Pico C SDK is not used.

## Setup (macOS)

```bash
rustup target add thumbv8m.main-none-eabihf
brew install picotool
```

## Flash and run

1. Hold **BOOTSEL** while plugging the board in (a drive named `RP2350` appears).
2. Build and flash (the cargo runner is `picotool load -u -v -x -t elf`):

   ```bash
   cargo run --release
   ```

3. Read the output (Ctrl-C to stop):

   ```bash
   cat /dev/cu.usbmodem*
   ```

```
Hello from Pico 2 W #0
Hello from Pico 2 W #1
...
```

## Notes

- After flashing, the `RP2350` drive disappears and the board shows up as a USB serial device. Re-enter BOOTSEL to flash again.
- The RP2350 boot ROM only starts an image that has an `IMAGE_DEF` block in the first 4 KiB of flash. `embassy-rp` emits it into `.start_block`, which `memory.x` places right after the vector table.
- Output starts only once the Mac opens the port (DTR). While waiting, the main loop sleeps with `Timer`. A loop with no real `.await` would starve the USB task, and the board would never see DTR.
- The onboard LED of the Pico 2 **W** is wired to the CYW43 Wi-Fi chip, not to a GPIO, so a plain GPIO blink needs an external LED.
