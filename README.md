# Autonomous Radio-Beacon Coordinator for Air Reconnaissance and Warning (ARCAN)

<img width="1587" height="871" alt="image" src="https://github.com/user-attachments/assets/13c5b611-b328-43ca-947d-a5b4db0bd168" />

## Software Usage

### Clean Target
```
cargo clean
```

### Build
```
cargo build --release
```
### Run
IMPORTANT: Pico has to be connected to your pc and the bootloader button on the mcu has to be press-held!
```
cargo run --release
```

### Log Data from Pico and Launch Terminal
```
sudo apt install tio
```
```
sudo tio /dev/ttyACM0
```
