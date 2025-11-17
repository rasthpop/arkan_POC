# Autonomous Radio-Beacon Coordinator for Air Reconnaissance and Warning (ARCAN)
<img width="1624" height="892" alt="image" src="https://github.com/user-attachments/assets/74267460-a938-4b69-ba5c-191813690dd2" />
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
