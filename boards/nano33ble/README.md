Arduino Nano 33 BLE
===================

<img src="https://store-cdn.arduino.cc/usa/catalog/product/cache/1/image/1040x660/604a3538c15e081937dbfbd20aa60aad/a/b/abx00031_featured.jpg" width="35%">

The [Arduino Nano 33 BLE](https://store.arduino.cc/usa/nano-33-ble) and [Arduino
Nano 33 BLE Sense](https://store.arduino.cc/usa/nano-33-ble-sense) are compact
boards based on the Nordic nRF52840 SoC. The "Sense" version includes the
following sensors:

- 9 axis inertial sensor
- humidity and temperature sensor
- barometric sensor
- microphone
- gesture, proximity, light color and light intensity sensor


## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md).

The Nano 33 comes pre-installed with a
[version](https://github.com/arduino/ArduinoCore-nRF528x-mbedos/tree/master/bootloaders/nano33ble)
of the BOSSA bootloader that Arduino uses on various boards. Unfortunately this
bootloader is not well suited for Tock development. Specifically, it doesn't
support reading from the board, so there is no way to automatically determine
what type of board it is or what is already installed. It's also [not open
source](https://github.com/arduino/ArduinoCore-nRF528x-mbedos/issues/23) (at
least as of December 2020).

For Tock development we need to replace the bootloader with the [Tock
Bootloader](htttps://github.com/tock/tock-bootloader). The Tock bootloader
allows a lot more flexibility with reading and writing the board, and is also
implemented on top of Tock itself.

This guide will walk through how to install the Tock bootloader, and describe
what is happening along the way. Our goal is to get the Tock bootloader flashed
to the nRF52840 at address 0x0 (overwriting the bossa bootloader). The bossa
bootloader does not have a mechanism for updating itself, however, so we have to
do this in a bit of a roundabout manner.

We also have a guide for restoring the BOSSA bootloader in case you want to go
back.

1. The first step is you will need the bossac tool. This tool is required to use
   the existing bootloader the board ships with.

    You can compile this tool from source:

	```shell
	$ git clone https://github.com/arduino/BOSSA
	$ cd BOSSA
	$ make bossac
	```

	Then you will need to add `BOSSA/bin` to your `$PATH` variable so that your
	system can find the `bossac` program.

2. Next we will use the bossa bootloader to load a copy of the Tock bootloader.
   The bossa bootloader expects that all application code (i.e. not the
   bootloader) starts at address 0x10000. That is, when the bootloader finishes
   it starts executing at address 0x10000.

    So, we will load a copy of the Tock bootloader to address 0x10000. That also means
    we need a version of the bootloader compiled to run at address 0x10000. This bootloader
    has already been compiled for you.

    To load the first Tock bootloader ensure the Nano 33 is in bootloader mode
    by double pressing the reset button (the light should pulse), and then:

    ```shell
    $ bossac -e -w bootloaders/tock-bootloader.v1.1.0.0x10000.bin
    ```

    Now the board should boot into the Tock bootloader.

3. Our last step is to use the temporary Tock bootloader to flash the real one
   at address 0x0. When the board boots it should run the Tock bootloader
   flashed at address 0x10000. To then flash the correct bootloader, run:

    ```shell
    $ tockloader flash bootloaders/tock-bootloader.v1.1.0.0x00000.bin --address 0
    ```

4. That's it! You now have the Tock bootloader. All `tockloader` commands should
   now work.

    You can test Tockloader by running:

    ```shell
    $ tockloader info
    ```

    You should see various properties of the board displayed.



## Programming the Kernel

You should be able to simply run `make program` in this directory
to install a fresh kernel.

```
$ make program
```

## Programming Applications

After building an application, you can use `tockloader install` to install it.

For example:

```shell
$ cd libtock-c/examples/blink
$ make
$ tockloader install
```

### Userspace Resource Mapping

This table shows the mappings between resources available in userspace
and the physical elements on the Nano 33 BLE board.

| Software Resource | Physical Element    |
|-------------------|---------------------|
| GPIO[2]           | Pin D2              |
| GPIO[3]           | Pin D3              |
| GPIO[4]           | Pin D4              |
| GPIO[5]           | Pin D5              |
| GPIO[6]           | Pin D6              |
| GPIO[7]           | Pin D7              |
| GPIO[8]           | Pin D8              |
| GPIO[9]           | Pin D9              |
| GPIO[10]          | Pin D10             |
| LED[0]            | Tri-color LED Red   |
| LED[1]            | Tri-color LED Green |
| LED[2]            | Tri-color LED Blue  |

## Debugging

The Nano 33 board uses a virtual serial console over USB to send debugging info
from the kernel and print messages from applications. You can use whatever your
favorite serial terminal program is to view the output. Tockloader also
supports reading and writing to a serial console with `tockloader listen`.

### Kernel Panics

If the kernel or an app encounter a `panic!()`, the panic handler specified in
`io.rs` is called. This causes the kernel to stop. You will notice the yellow
LED starts blinking in a repeating but slightly irregular pattern. There is also
a panic print out that provides a fair bit of debugging information. That panic
output is output over the USB CDC connection and so should be visible as part of
the output of `tockloader listen`, however if your kernel panics so early that
the USB connection has not yet been established you will be unable to view any
panic output. In this case, you can modify the panic handler to instead output
panic information over the UART pins, but you will have to separately interface
with the UART pins on the board in order to observe the serial output.

## Factory Reset

To restore the BOSSA bootloader we can largely reverse the steps used to install
the Tock bootloader.

1. First we need to install a temporary copy of the Tock bootloader.

    ```shell
    $ tockloader flash bootloaders/tock-bootloader.v1.1.0.0x10000.bin --address 0x10000
    ```

2. Now we can restore the BOSSA bootloader.

    ```shell
    $ tockloader flash bootloaders/bossa.0x00000.bin --address 0x0
    ```

    Double clicking reset should enter the bossa bootloader now.
