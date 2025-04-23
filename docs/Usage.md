# Clockit Usage Guide

Clockit is a simple yet powerful terminal-based timer with ASCII art display.

## Basic Commands

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

### Pomodoro Timer

```bash
# Start a Pomodoro timer (25min work, 5min break cycles)
clockit -p
# or
clockit --pomodoro
```

### Configuration Initialization

To create a default configuration file:

```bash
clockit --init-config
```

## Time Format

The countdown timer accepts time in several formats:

- `SS` - Seconds only (e.g., `90`)
- `MM:SS` - Minutes and seconds (e.g., `5:30`)
- `HH:MM:SS` - Hours, minutes, and seconds (e.g., `2:30:15`)

Clockit handles overflow automatically. For example, `0:90:70` becomes `1:31:10` (90 minutes = 1 hour 30 minutes, 70 seconds = 1 minute 10 seconds).

## Controls

When the timer is running:

- Press `q` to quit
- Press `Ctrl+C` to exit
- During Pomodoro mode, press any key to proceed to the next session

## Configuration

Clockit can be configured through a YAML configuration file. See the [sample configuration](sample-config.yaml) for details.

The configuration file is located at:
- Linux/macOS: `~/.config/clockit/config.yaml`
- Windows: `%APPDATA%\clockit\config.yaml`

### Configuration Options

- **Colors**: Change the display colors for different elements
- **Blink Separator**: Toggle the blinking of colons and dots
- **Refresh Rates**: Adjust update frequency for smoother display

To apply configuration changes, simply edit the file and restart Clockit.

## Troubleshooting

If the timer display appears distorted or has alignment issues:
1. Check if your terminal uses a monospaced font
2. Try adjusting your terminal window size
3. Ensure your terminal supports the colors specified in your configuration

For configuration issues, try regenerating the default configuration:
```bash
rm ~/.config/clockit/config.yaml  # Linux/macOS
# or
del %APPDATA%\clockit\config.yaml  # Windows

clockit --init-config
```