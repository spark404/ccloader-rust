ccloader-rust
===

Small tool to upload firmware to an CC2531 USB stick using an
Arduino.

The source code for the Arduino sketch is in the CCLoader directory.
See the accompanied README.md for wiring the debug connector to an
Arduino.

The loader application itself takes the following arguments:

```text
Usage: ccloader-rust [OPTIONS] --port <PORT> --firmware <FIRMWARE>

Options:
  -p, --port <PORT>          Serial port
  -f, --firmware <FIRMWARE>  The firmware file to upload
      --verify               Verify all uploads
  -h, --help                 Print help
  -V, --version              Print version
```

This is all based on the work done here: https://github.com/RedBearLab/CCLoader
