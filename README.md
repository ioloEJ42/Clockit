# Clockit

A beautiful terminal-based timer with ASCII art display and customizable configuration.

![Clockit Demo](demo.gif)

## Features

- Large, easy-to-read ASCII art digits
- Colorful terminal interface with customizable colors
- Both countdown timer and stopwatch functionality
- Multiple time formats with overflow handling
- Configurable visual options
- Simple keyboard controls

## Installation

### From Source

1. Make sure you have Rust installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:
   ```bash
   git clone https://github.com/ioloej42/clockit
   cd clockit
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. Install the binary:
   ```bash
   sudo cp target/release/clockit /usr/local/bin/
   ```

## Configuration

Clockit uses a YAML configuration file to customize colors and behavior.

### Creating the default configuration

You can create the default configuration file by running:

```bash
clockit --init-config
```

This will create a configuration file at:
- Linux/macOS: `~/.config/clockit/config.yaml`
- Windows: `C:\Users\<username>\AppData\Roaming\clockit\config.yaml`

### Customization Options

- **Colors**: Change the color of the clock digits, instructions, and time's up message
- **Refresh Rate**: Adjust how frequently the timer updates
- **Separator Blinking**: Enable/disable blinking of the colons and dots

See the [sample configuration](docs/sample-config.yaml) for more details.

## Usage

### Countdown Timer

```bash
# Start a 5-minute timer
clockit -c 5:00

# Start a 90-second timer
clockit -c 1:30
# or
clockit -c 90

# Start a 2-hour, 30-minute, and 15-second timer
clockit -c 2:30:15

# Timer handles overflow
clockit -c 0:75:90  # Same as 1:16:30
```

### Stopwatch

```bash
# Start a stopwatch
clockit -s
```

### Controls

- Press `q` or `Ctrl+C` to exit at any time

## License

MIT