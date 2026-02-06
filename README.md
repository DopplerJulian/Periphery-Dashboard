# Peripheral Dashboard for Raspberry Pi Pico 2 W

This project aims to provide the firmware for an E-ink dashboard that, in conjunction with a local hub, can display images transmitted on a regular basis.

## Hardware Requirements

- Raspberry Pi Pico 2 W
- E-ink display (Waveshare 7.5" 3 Color E-ink display)
- Debug probe (Optional)

## Software Tools

- Rust, with the ``thumbv8m.main-none-eabihf`` target
- picotool
- probe-rs (For debugging)

## Roadmap

### Scaffolding

- [ ] Setup initial project structure
- [ ] Setup Bluetooth LE (TrouBLE + cyw43)
- [ ] Setup E-ink display (find a suitable driver or implement a basic one)

### MVP

- [ ] Configure TrouBLE peripheral service
- [ ] Link TrouBLE with E-ink driver
- [ ] Set splash screen at (pre pairing)

### Future

- [ ] Configure passkey or numeric comparison pairing
- [ ] Implement image decompression 
- [ ] Override a segment of the display relevant information (e.g. last image received, last connection to hub, current firmware version)
- [ ] Optimize for energy efficiency
