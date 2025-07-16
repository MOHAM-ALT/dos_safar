# DOS Safar - Universal ARM Boot Manager

## Project Overview

**DOS Safar** is a lightweight, intelligent boot manager designed for ARM-based devices including Raspberry Pi, gaming handhelds, and ARM development boards. It solves the common problem of display and input configuration issues that occur when switching between different operating systems on ARM devices.

## Problem Statement

When using multiple operating systems on ARM-based devices (gaming handhelds, Raspberry Pi, development boards) with attached displays and input devices, users frequently encounter:
- Display configuration issues after OS switching
- Input device mapping problems requiring manual reconfiguration  
- Gaming controller compatibility issues across different systems
- Need to physically access the device for troubleshooting
- Time-consuming setup process for each OS switch
- Retro gaming emulator configuration conflicts

## Solution

DOS Safar provides:
1. **Hardware Testing Suite**: Automatic testing of display, keyboard, and network on boot
2. **Configuration Persistence**: Saves and applies working hardware configurations across OS switches
3. **Multi-OS Management**: Easy installation, removal, and switching between operating systems
4. **Remote Access**: Automatic network connection with web-based remote control interface
5. **Quick Deployment**: Fast imaging and flashing tools for rapid testing and deployment

## Key Features

### 🔧 Hardware Management
- **Display Testing**: Automatic resolution detection and configuration persistence (HDMI, DSI, small LCD screens)
- **Input Device Testing**: Gaming controllers, keyboards, touchscreens, and d-pad validation
- **Network Testing**: WiFi, Ethernet connectivity verification and automatic connection
- **Gaming Hardware**: Controller mapping, audio output selection, performance profiling
- **Device Detection**: Automatic identification of ARM device type and capabilities
- **Configuration Storage**: Persistent storage of working hardware settings across reboots

### 💾 Multi-OS Support
- **OS Detection**: Automatic scanning and cataloging of installed systems (RetroPie, Batocera, Raspberry Pi OS, Ubuntu, etc.)
- **Boot Menu**: Clean, gaming-friendly selection interface optimized for small screens
- **Gaming OS Support**: Pre-configured profiles for retro gaming distributions
- **OS Installation**: Remote installation of new operating systems and gaming images
- **OS Removal**: Safe removal of unused systems with data backup
- **Partition Management**: Intelligent disk space allocation for multiple gaming systems

### 🌐 Remote Management
- **Auto-Connect**: Automatic network connection after 3-second timeout
- **Mobile-Friendly Interface**: Touch-optimized web interface accessible from phones/tablets
- **Screen Sharing**: View ARM device display remotely (perfect for handheld gaming devices)
- **Remote Control**: Full input control via web interface (virtual d-pad, buttons)
- **File Transfer**: Upload/download ROMs, saves, and OS images remotely
- **Gaming Features**: ROM management, save state backup, controller configuration

### 🚀 Development Tools
- **Image Builder**: Create custom DOS Safar images
- **Flash Tool**: Quick deployment to SD cards/USB drives
- **Configuration Manager**: Easy setup and customization

## Technical Specifications

### Target Platform
- **Primary**: Raspberry Pi (4B, 4B+, 5, 3B, 3B+, Zero 2W)
- **Gaming Handhelds**: Anbernic (RG351, RG552, RG35XX), Powkiddy (RGB10, RGB30), Miyoo Mini+, Retroid Pocket series
- **ARM Development Boards**: Orange Pi, Rock Pi, Banana Pi, ODROID, Jetson Nano
- **Android Gaming Devices**: GPD XP+, AYN Odin, Steam Deck (ARM compatibility)
- **Architecture**: ARM64/ARMv7/ARMv8

### Programming Language
- **Rust** - Chosen for performance, memory safety, and excellent cross-compilation support

### Key Dependencies
- **Hardware Interface**: rppal (Raspberry Pi Peripheral Access Library)
- **Web Server**: axum (fast, ergonomic web framework)
- **System Interface**: nix (Unix system interface)
- **Configuration**: toml (configuration file format)
- **Async Runtime**: tokio (asynchronous runtime)

### Memory Requirements
- **Boot Manager**: ~64MB RAM minimum
- **Storage**: ~256MB for boot partition
- **Network**: WiFi or Ethernet connection

## Project Structure

```
dos_safar/
├── src/
│   ├── hardware/          # Hardware testing and configuration
│   ├── bootloader/        # Multi-OS boot management
│   ├── remote/           # Web interface and remote control
│   └── utils/            # Shared utilities and configuration
├── tools/                # Development and deployment tools
├── config/               # Configuration templates
├── assets/               # Static assets and web UI
└── docs/                 # Documentation and guides
```

## Use Cases

### 1. Retro Gaming Enthusiasts
- Switch between different retro gaming distributions (RetroPie, Batocera, Recalbox)
- Backup and restore game saves across different OS installations
- Remote ROM management and organization from PC/phone
- Controller configuration persistence across systems

### 2. Gaming Handheld Users
- Manage multiple OS installations on devices like Anbernic RG351 or Miyoo Mini
- Remote troubleshooting without disassembling device
- Easy firmware updates and custom OS installation
- Performance optimization for different game systems

### 3. ARM Development and Testing
- Developers testing applications across multiple ARM OS environments
- Remote debugging and development without physical device access
- Rapid deployment of test images and configurations
- Cross-platform compatibility testing

### 4. Educational and Maker Projects
- Students experimenting with different ARM operating systems
- Teachers remotely managing multiple ARM devices in classroom
- Maker projects requiring easy OS switching for different functionalities
- IoT prototyping with multiple system configurations

## AI Assistant Instructions

When working with this project, please note:

### Project Context
- This is a **Universal ARM device** boot manager with focus on **gaming handhelds and Raspberry Pi**
- The primary goal is **solving hardware configuration persistence** across OS switches on ARM devices
- **Gaming-focused features** are essential - controller support, small screen optimization, ROM management
- **Remote access** is a key feature but **security is not a primary concern** for this project
- The user wants **quick, practical solutions** over complex security implementations
- **Flash Tool**: A Windows-based Python GUI application is already completed for flashing images

### Development Approach
- Prioritize **functionality and simplicity** over security features
- Focus on **ARM hardware specifics** (GPIO, display interfaces, gaming controls, small screens)
- Use **Rust or Python** with best practices but keep code readable and maintainable
- **Auto-network connection** after 3-second timeout is a key requirement
- **Gaming device optimization** - support for d-pads, small screens, battery management
- **Device auto-detection** - automatically identify ARM device type and configure accordingly
- **Minimal security**: Simple web interface without complex authentication
- **Fast development**: Prefer working solutions over perfect architecture

### Code Style and Language Requirements
- **ALL CODE must be written in ENGLISH ONLY** - variable names, comments, function names, documentation
- **Modular design** with clear separation of concerns
- **Error handling** should be practical, not exhaustive
- **Configuration files** in TOML format for easy editing
- **Async/await** for network operations and I/O
- **Clear naming conventions** using descriptive English terms
- **Simple, readable code** over clever optimizations

### Testing and Deployment
- Provide **quick testing methods** using development tools
- **Image creation** should be simple and fast
- **Cross-compilation** setup for ARM targets
- **Remote debugging** capabilities are valuable
- **Use existing flash tool** for rapid deployment and testing
- **Emulation support** for development without real hardware

### Features to Prioritize
1. **Hardware testing and configuration persistence** (display, input devices, gaming controls)
2. **Device auto-detection** (Raspberry Pi, Anbernic, Orange Pi, etc.)
3. **Gaming-optimized boot menu** (small screen friendly, controller navigation)
4. **Mobile-friendly web interface** (touch controls, responsive design)
5. **Multi-OS management** (RetroPie, Batocera, standard Linux distributions)
6. **Gaming features** (ROM management, save states, controller mapping)
7. **Image building and flashing tools** (integrate with existing Windows tool)

### Features to Avoid
- Complex authentication systems
- Advanced security features
- Unnecessary abstractions
- Over-engineered solutions
- Non-English code or comments
- Complex configuration systems

### Development Tools Available
- **Windows Flash Tool**: Python-based GUI application for image flashing
- **Cross-compilation**: Rust toolchain for ARM targets
- **Quick Testing**: Use QEMU or direct hardware testing
- **Remote Access**: Web-based interface for development and debugging

### Hardware Specifications to Consider
- **Target Devices**: Raspberry Pi (all models), Gaming handhelds (Anbernic, Powkiddy, Miyoo), ARM dev boards
- **Memory Constraints**: 64MB RAM minimum for boot manager (optimize for low-end gaming devices)
- **Storage**: 256MB boot partition minimum, SD card and eMMC support
- **Display Support**: HDMI, DSI, small LCD panels (320x240 to 1920x1080), touch screens
- **Input Support**: Gaming controllers (USB, Bluetooth), d-pads, analog sticks, keyboards, touch input
- **Network**: WiFi and Ethernet support with gaming handheld antenna considerations
- **GPIO**: Hardware testing and configuration interfaces for various ARM platforms
- **Audio**: Multiple output options (3.5mm, HDMI, I2S, internal speakers)
- **Power Management**: Battery monitoring for handheld devices, sleep/wake functionality

## Quick Build and Flash

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM target
rustup target add armv7-unknown-linux-gnueabihf

# Install cross-compilation tools
sudo apt install gcc-arm-linux-gnueabihf
```

### Development Workflow
```bash
# Clone the repository
git clone https://github.com/yourusername/dos_safar.git
cd dos_safar

# Build for Raspberry Pi
cargo build --release --target armv7-unknown-linux-gnueabihf

# Create bootable image
cargo run --bin image_builder

# Use Windows Flash Tool for deployment
# 1. Copy generated image to Windows machine
# 2. Run DOS Safar Flash Tool (Python GUI)
# 3. Select image and target SD card
# 4. Flash and test on Raspberry Pi
```

### Quick Testing Setup
```bash
# Test on QEMU (for development)
qemu-system-arm -M versatilepb -cpu arm1176 -m 256 \
  -kernel target/armv7-unknown-linux-gnueabihf/release/dos_safar \
  -serial stdio -nographic

# Test web interface locally
cargo run --bin web_server_test
# Open browser: http://localhost:8080
```

## Roadmap

### Phase 1: Core Functionality (PRIORITY)
- [ ] Hardware testing suite (display, keyboard, network configuration)
- [ ] Configuration persistence system (save/restore settings)
- [ ] Basic boot menu with 3-second timeout
- [ ] Image building tools integration
- [ ] Auto-network connection and web interface startup

### Phase 2: Multi-OS Management  
- [ ] OS detection and cataloging system
- [ ] Installation and removal workflows
- [ ] Partition management utilities
- [ ] Boot configuration management

### Phase 3: Remote Access Enhancement
- [ ] Web interface for remote control
- [ ] Screen sharing/VNC-like functionality  
- [ ] Remote file transfer system
- [ ] System monitoring dashboard

### Phase 4: Polish and Production
- [ ] Performance optimizations
- [ ] Comprehensive error handling
- [ ] Documentation and user guides
- [ ] Hardware compatibility testing

## Contributing

This project is designed to work closely with AI assistants. When contributing:

1. **Keep it simple** - Avoid over-engineering
2. **Focus on Raspberry Pi** - Hardware-specific optimizations are welcome
3. **Document for AI** - Clear comments and structure help AI understanding
4. **Test on real hardware** - Emulation is not sufficient for this project

## License

MIT License - Feel free to use, modify, and distribute as needed.

## Support

This project is designed to be AI-assistant friendly. When asking for help:
- Provide specific hardware details (Pi model, display type, etc.)
- Include error messages and logs
- Specify what you're trying to achieve
- Mention if you're testing on real hardware or emulation

---

**DOS Safar** - Making ARM device OS management simple and accessible for gaming, development, and embedded projects.