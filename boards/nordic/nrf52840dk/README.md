Platform-Specific Instructions: nRF52840-DK
===================================

The [nRF52840 Development
Kit](https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK) is a platform
based around the nRF52840, an SoC with an ARM Cortex-M4 and a BLE
radio. The kit is Arduino shield compatible and includes several
buttons.

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has an
integrated JTAG debugger, you simply need to [install JTAG
software](../../../doc/Getting_Started.md#loading-the-kernel-onto-a-board).

## Programming the kernel
Once you have all software installed, you should be able to simply run
make flash in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application over USB using the integrated JTAG and `tockloader`:

```bash
$ cd libtock-c/examples/<app>
$ make
$ tockloader install --jlink --board nrf52dk
```

The same options (`--jlink --board nrf52dk`) must be passed for other tockloader commands
such as `erase-apps` or `list`.

Viewing console output on the nrf52840dk is slightly different from other boards. You must use
```bash
$ tockloader listen
```
**followed by a press of the reset button** in order to view console output starting from the boot
sequence. Notably, you should not
pass the `--jlink` option to `tockloader listen`.

## Console output

This board supports two methods for writing messages to a console interface
(console driver for applications as well as debug statements in the kernel).

By default, messages are written to a UART interface over the GPIO pins `P0.05`
to `P0.08` (see the [main.rs](src/main.rs) file).

If you don't have any UART cables or want to use a different interface, there is
also a console over the Segger RTT protocol. This only requires a micro-USB
cable on the USB debugging port (the same used to flash Tock on the board), and
is enabled by setting the `USB_DEBUGGING` constant to `true` in the
[main.rs](src/main.rs) file.
This disables the UART interface.

For instructions about how to receive RTT messages on the host, see the
[corresponding capsule](../../../capsules/src/segger_rtt.rs).

## Debugging

See the [nrf52dk README](../nrf52dk/README.md) for information about debugging
the nRF52840dk.

## Bootloader stuff

1. clone adafruit bootloader

2. `make BOARD=pca10056 flash` will install the adafruit bootloader

3. Then flash the tock bootloader

4. Get a bin of the adafruit bootloader



## adafruit -> tock

adafruit-nrfutil --verbose dfu serial -pkg bootloader-0xf4000.zip -p /dev/cu.usbmodem14101 -b 115200 --singlebank

<RESET>

tockloader flash ~/git/tock-bootloader/target/thumbv7em-none-eabi/release/nrf52-cdc-bootloader-0x0.bin  --address 0



## tock -> adafruit

tockloader flash ~/git/tock-bootloader/target/thumbv7em-none-eabi/release/nrf52-cdc-bootloader-0x10000.bin  --address 0x10000



cd Adafruit_nRF52_Bootloader/_build/build-pca10056

arm-none-eabi-objcopy -O binary --remove-section .uicrBootStartAddress --remove-section .uicrMbrParamsPageAddress --gap-fill 0xff pca10056_bootloader-0.3.2-190-geab7e68.out pca10056_bootloader-0.3.2-190-geab7e68.bin





tockloader flash /Users/bradjc/git/Adafruit_nRF52_Bootloader/_build/build-pca10056/pca10056_bootloader-0.3.2-190-geab7e68.bin --address 0xf4000



tockloader write 0xff000 0xff 512


// pad mbr.bin to 4096 bytes with 0xff


tockloader flash /Users/bradjc/git/Adafruit_nRF52_Bootloader/lib/softdevice/mbr/hex/mbr_nrf52_2.4.1_mbr.bin  --address 0x0 --pad 1280 0xff










## bossa -> tock

bossac -w ~/git/tock-bootloader/target/thumbv7em-none-eabi/release/nrf52-cdc-bootloader-0x10000.bin -p /dev/cu.usbmodem14101


tockloader flash ~/git/tock-bootloader/target/thumbv7em-none-eabi/release/nrf52-cdc-bootloader-0x0.bin  --address 0



## tock -> bossa

tockloader flash ~/git/tock-bootloader/target/thumbv7em-none-eabi/release/nrf52-cdc-bootloader-0x10000.bin  --address 0x10000

tockloader flash ~/git/ArduinoCore-nRF528x-mbedos/bootloaders/nano33ble/bootloader.bin --address 0
