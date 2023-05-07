CCLoader
========

Original source: https://github.com/RedBearLab/CCLoader

This is a slightly modified version that adds more
standard crc check. Designed to work with the rust 
ccloader.

To wireup the Arduino:

| CC2531 debug pin | Arduino     |
|------------------|-------------|
| VCC              | 3.3v vout   |
| GND              | GND         |
| DD               | D6          |
| DC               | D5          |
| Reset            | D4          |

Note that connecting the 3.3v is optional when the
device is powered by USB.

This version has been tested with a CC2531 USB stick and an Arduino Leonardo:

`ID 0451:16a8 Texas Instruments, Inc. CC2531 ZigBee`