# Robodog

Building drivers to interact with the hardware compatible with https://github.com/Freenove/Freenove_Robot_Dog_Kit_for_Raspberry_Pi

## Drivers

Drivers are implemented using `embedded-hal`.
Users of the drivers should use `linux_embedded_hal` to provide the i2c implementaion.


## Build

Add the toolchain for the pi
```
rustup target add aarch64-unknown-linux-gnu
```
the approproate gcc
```
sudo apt-get install gcc-aarch64-linux-gnu libclang-dev
```
and then build
```bash
RUSTFLAGS="-C linker=aarch64-linux-gnu-gcc" \
    cargo build --target=aarch64-unknown-linux-gnu --release
```


## Setup Robodog

Ensure that `/boot/firmware/config.txt` contains:
```txt
dtparam=i2c_arm=on,i2c_arm_baudrate=400000
# dtparam=audio=on
```

and that ` /etc/modprobe.d/snd-blacklist.conf` contains:
```
blacklist snd_bcm2835
```
