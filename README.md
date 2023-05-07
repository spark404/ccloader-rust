ccloader-rust
===


A command-line interface (CLI) program written in Rust, designed to facilitate the uploading of firmware to cc2531 USB 
sticks. This program is particularly useful for developers and engineers who work with these USB sticks and need a 
quick and easy way to upload firmware to them. 

CCLoader
---
To ensure that ccloader-rust works correctly, it requires the CCLoader sketch to be uploaded to an Arduino board 
beforehand. The sketch in question is responsible for enabling the Arduino board to act as a USB-to-debug adapter. 
Without this firmware, ccloaderr-rust will not be able to communicate with the cc2531 debugger interface properly.


The source code for the CCLoader sketch is in the CCLoader directory.
See the accompanied [README.md](CCLoader/README.md) for wiring the debug interface to an
Arduino.

CLI
---
The ccloader-rust utility itself takes the following arguments:

```text
Usage: ccloader-rust [OPTIONS] --port <PORT> --firmware <FIRMWARE>

Options:
  -p, --port <PORT>          Serial port
  -f, --firmware <FIRMWARE>  The firmware file to upload
      --verify               Verify all uploads
  -h, --help                 Print help
  -V, --version              Print version
```

Note that the serial port mentioned is the serial port of the Arduino, which is most likely the same port that the 
sketch was uploaded to.

Thanks
--
This is all based on the work done here: https://github.com/RedBearLab/CCLoader
