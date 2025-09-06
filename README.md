# Orange Pi Control

## Overview

This project is the control system for the Orange Pi. It handles:

* Communication with a Rust backend (HTTP API)
* USB-C serial communication with ESP32 using JSON messages
* Capturing images from a microscope camera
* Processing images using OpenCV

It is written in **Rust** and designed to run on Linux (Ubuntu). The code is modular so hardware-specific modules can be replaced easily once the Orange Pi is available.

---

## Project Structure

```
src/
├── main.rs             # Entry point, orchestrates async tasks
├── backend.rs          # Handles HTTP communication with backend
├── esp32.rs            # Serial communication with ESP32
├── camera.rs           # Captures images from microscope camera
├── image_processing.rs # Processes captured images (OpenCV)
└── models.rs           # Shared message structs
```

---

## Dependencies

### Rust
s
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
cargo --version
```

### System Packages

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libopencv-dev
sudo apt install -y serialport-tools  # optional for testing serial
```

### Cargo Dependencies

See `Cargo.toml`:

* tokio
* serde + serde\_json
* reqwest
* serialport
* opencv
* anyhow

---

## Setup & Run

1. Clone the repo (or work in your VM):

```bash
git clone <your-repo-url>
cd orangepi-control
```

2. Build the project:

```bash
cargo build
```

3. Run the project:

```bash
cargo run
```

* The app will start async tasks for backend and ESP32 communication.

---

## Notes

* **ESP32 Communication:** Make sure the ESP32 is connected over USB-C, or mock it if not available.
* **Backend:** Update the URL in `backend.rs` to point to your running Rust backend.
* **Hardware Abstraction:** GPIO, motors, and microscope camera modules can be mocked for testing on a VM.

---

