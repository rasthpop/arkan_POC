# Autonomous Radio-Beacon Coordinator for Air Reconnaissance and Warning (ARCAN)
<img width="1475" height="881" alt="image" src="https://github.com/user-attachments/assets/0fef5989-b67a-48fd-a780-615ce4ae5fee" />

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
