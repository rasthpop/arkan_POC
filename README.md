# Autonomous Radio-Beacon Coordinator for Air Reconnaissance and Warning (ARCAN)

<img width="1331" height="802" alt="image" src="https://github.com/user-attachments/assets/51e30d10-1cbe-406a-bf89-ec0cc512b014" />


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
