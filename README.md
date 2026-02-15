## Purpose & Context

**BSManager** runs on an Orange Pi device located local to a Remote Controlled Microscope we call BioScope.  
It serves as a core component of the **BioScope prototype system**, which enables fully remote operation of microscopes for students.

### Purpose
BSManager handles **real-time device control**, **sensor and camera data acquisition**, and **secure messaging**, ensuring reliable operation in distributed environments.

### Context in the Bioscope System
- **ESP32 & Hardware Layer:** Communicates with ESP32 modules, motors, and other hardware components to control microscope movement and sensors.  
- **Backend Layer:** Streams processed data and receives commands from the Bioscope backend, allowing centralized orchestration of the microscope and other devices.  
- **Frontend Layer:** Feeds data to user dashboards, enabling remote monitoring and control of the microscope in real time.

Final Product

![Embedded Frontend Controls](https://github.com/user-attachments/assets/39a01845-bf39-4ae9-90d3-b65516f8dba0)

![Full Hardware Setup)](https://github.com/user-attachments/assets/0f5714c2-5e27-4786-b4b7-f1bcba7789ae)

## Features
- Async runtime powered by **Tokio**
- Secure TLS + WebSocket communications via **rustls** and **tokio-tungstenite**
- Serial communication using **tokio-serial**
- Structured message serialization via **serde** and **serde_json**
- Timekeeping and unique IDs using **chrono** and **uuid**
- Cryptography primitives from **ring**
- Optional camera support via **OpenCV** and **V4L2**
- Easy `.env` configuration using **dotenv**
- Compatible with **RISC-V Linux** cross-compilation
---
## System Requirements

### Hardware
- Orange Pi board running Linux (e.g. Orange Pi RV2 / R1 Plus / 5 / 5B)
- Network connectivity (Ethernet or Wi-Fi)
- Optional USB camera (for OpenCV/V4L modules)
- Optional UART adapter (for serial devices)

### Software on Orange Pi
Install the base packages needed for building and running the program:

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  libopencv-dev \
  libclang-dev \
  libssl-dev \
  cmake \
  clang \
  git
```

---

## Rust Setup

### 1. Install Rust
If not installed yet:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Add the RISC-V target (for cross-compiling)
For a **RISC-V 64-bit Linux** target:
```bash
rustup target add riscv64gc-unknown-linux-gnu
```

You may need a cross-compiler toolchain:
```bash
sudo apt install -y gcc-riscv64-linux-gnu
```

---

## Building

### Native build (on the Orange Pi)
```bash
cargo build --release
```
Binary will appear at:
```
target/release/orangepi-IA
```

### Cross-compiling on a PC (for RISC-V)
```bash
cargo build --release --target riscv64gc-unknown-linux-gnu
```
Binary will appear at:
```
target/riscv64gc-unknown-linux-gnu/release/orangepi-IA
```
Copy to Orange Pi:
```bash
scp target/riscv64gc-unknown-linux-gnu/release/orangepi-IA orangepi@<pi-ip>:/home/orangepi/
```

---

## Running

### Run with debug logging
```bash
RUST_LOG=info ./orangepi-IA
```

### Or just:
```bash
cargo run
```

### Optional `.env` configuration
Create a `.env` file in the project root:

```dotenv
# Network configuration
SERVER_HOST="bsdapidev.webschool.au"
SERVER_PORT="443"

# Device identity
DEVICE_NAME="KYRIE IRVING"

# Authentication token (Base64 encoded)
AUTH_TOKEN="cGxhbm5pbmdsdW5jaGNvbnRyYXN0cGF0aGRpcmVjdGx5ZmxhZ2F3YXJlc29hcG1vdmk="

# Serial and camera settings (optional)
SERIAL_PORT=/dev/ttyUSB0
CAMERA_INDEX=0

# Logging
LOG_LEVEL=info
```

> **Note:** Keep `.env` secret if it contains real authentication tokens.

---

## Useful Commands

| Command | Description |
|----------|-------------|
| `cargo check` | Check syntax and dependencies |
| `cargo run` | Build and run in debug mode |
| `cargo build --release` | Optimized build |
| `cargo clean` | Clean build artifacts |
| `git log --oneline` | View commit history |

---

## Troubleshooting

### `linker cc not found`
```bash
sudo apt install -y build-essential
```

### `opencv` build errors
```bash
sudo apt install -y libopencv-dev pkg-config
```

### Serial permission denied
```bash
sudo usermod -a -G dialout $USER
sudo reboot
```

### Cross-compile linker errors
```bash
cat >> .cargo/config.toml <<EOF
[target.riscv64gc-unknown-linux-gnu]
linker = "riscv64-linux-gnu-gcc"
EOF
```
Then rebuild:
```bash
cargo build --release --target riscv64gc-unknown-linux-gnu
```

---

## License
MIT License 2025 Kevin La

---

## Credits
Developed by **Kevin La** and **Team Denmark**  
Rust + Orange Pi integration with focus on secure async communication and embedded system reliability.
