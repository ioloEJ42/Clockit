# Installation Guide for Clockit

This guide covers how to install Clockit on different operating systems.

## Prerequisites

Clockit requires:
- Rust (version 1.56.0 or later)
- A terminal that supports ANSI colors

## Installing Rust

If you don't have Rust installed, you can install it using rustup:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run rustup-init.exe from https://rustup.rs
```

## Installation Methods

### From Source (Recommended)

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/clockit.git
   cd clockit
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the binary:
   ```bash
   # Linux/macOS
   sudo cp target/release/clockit /usr/local/bin/
   
   # Windows (Run in PowerShell as Administrator)
   copy target\release\clockit.exe C:\Windows\System32\
   ```

### Using Cargo

If you prefer, you can install directly using Cargo:

```bash
cargo install --git https://github.com/yourusername/clockit.git
```

The binary will be installed to `~/.cargo/bin/` on Linux/macOS or `%USERPROFILE%\.cargo\bin\` on Windows.

## Verifying Installation

To verify that Clockit is installed correctly:

```bash
clockit --init-config
```

This should create a default configuration file and display a message indicating where it was created.

## First Run

After installation, create a default configuration file:

```bash
clockit --init-config
```

Then, try running a simple timer:

```bash
clockit -c 10
```

You should see a countdown timer with ASCII art digits appearing in your terminal.

## Uninstallation

To uninstall Clockit:

```bash
# If installed manually
sudo rm /usr/local/bin/clockit  # Linux/macOS
rm C:\Windows\System32\clockit.exe  # Windows

# If installed via Cargo
cargo uninstall clockit
```

## Troubleshooting

### Command Not Found

If you encounter a "command not found" error:

1. Make sure the installation directory is in your PATH
2. On Linux/macOS, check file permissions: `chmod +x /usr/local/bin/clockit`
3. Try using the full path to the executable

### Terminal Support Issues

If you see garbled output or color issues:

1. Ensure your terminal supports ANSI colors
2. Try a different terminal application
3. Edit your configuration to use basic colors (black, white, red, green, etc.)