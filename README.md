# Daisy AI

Figure out which board you have:
- Daisy Seed (codec AK4556), seed
- Daisy Seed 1.1 (codec WM8731), seed_1_1
- Daisy Seed 1.2 (codec PCM3060), seed_1_2
- Daisy Patch SM (codec PCM3060), patch_sm

then change the feature in the `Cargo.toml` to your board.

```toml
daisy = { version = "0.11", features = ["seed_1_1"] }
```

## Flash Firmware

### 1. Flash Bootloader

First, install the bootloader on the board. You can use <https://flash.daisy.audio/>,
go to the "Bootloader" tab, select version "v6.2", and flash it. Alternatively
you can use the [libDaisy](https://github.com/electro-smith/libDaisy/tree/master)
project and its `Makefile`.

Once the bootloader is installed, restart the module and press the BOOT button
within the first 2 seconds after startup. The onboard LED should start pulsing,
indicating the bootloader is active and waiting.

### 2. Build and flash the program

```sh
cargo objcopy --release --bin daisy-ai -- -O binary target/program.bin
dfu-util -a 0 -s 0x90040000:leave -D target/program.bin -d ,0483:df11
```

## Attach Debug Probe
```sh
probe-rs attach --chip STM32H750VBTx --protocol swd target/thumbv7em-none-eabihf/release/daisy-ai
```

## Environment Setup Fedora

```sh
sudo dnf install libusbx-devel libftdi-devel libudev-devel
# Install probe-rs
curl -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
# Flip-link helps with stack overflow
cargo install flip-link
```

### Enable USB device access

```sh
sudo nano /etc/udev/rules.d/50-stm32-dfu.rules
```

### Add this content

```sh
# STM32 DFU Device (Daisy Seed)
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="df11", MODE="0666", GROUP="plugdev"
# ST-Link Mini v3
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3754", MODE:="0666", SYMLINK+="stlinkv3_%n"
```

### Add user to groups

```sh
# Add your user to dialout and plugdev groups
sudo usermod -a -G plugdev $USER

# Check if groups exist, create if needed
getent group plugdev || sudo groupadd plugdev
```

### Reload user groups

```sh
# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

More information: [https://probe.rs/docs/getting-started/probe-setup/](https://probe.rs/docs/getting-started/probe-setup/)
