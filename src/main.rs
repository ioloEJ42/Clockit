// src/main.rs
mod config;
mod digit;

use clap::Parser;
use config::Config;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Color, Stylize},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use std::{
    io::{self, stdout, Write},
    thread,
    time::{Duration, Instant},
};

/// A beautiful ASCII art timer for the terminal
#[derive(Parser)]
#[command(name = "clockit")]
#[command(about = "A beautiful ASCII art timer for the terminal", long_about = None)]
struct Cli {
    /// Start a countdown timer in HH:MM:SS format
    #[arg(short = 'c', long = "countdown")]
    countdown: Option<String>,

    /// Start a stopwatch
    #[arg(short = 's', long = "stopwatch", default_value_t = false)]
    stopwatch: bool,
    
    /// Start a Pomodoro timer (default: 25min work, 5min break)
    #[arg(short = 'p', long = "pomodoro", default_value_t = false)]
    pomodoro: bool,
    
    /// Generate a default config file
    #[arg(long = "init-config", default_value_t = false)]
    init_config: bool,
}

/// Parse a time string in format "HH:MM:SS" or "MM:SS" or "SS"
/// Handles overflow in any position (e.g., 75 seconds becomes 1 minute 15 seconds)
fn parse_time_string(time_str: &str) -> Result<u64, &'static str> {
    let parts: Vec<&str> = time_str.split(':').collect();
    
    // Initialize counters for hours, minutes, seconds
    let mut hours = 0;
    let mut minutes = 0;
    let mut seconds;
    
    match parts.len() {
        // Just seconds
        1 => {
            seconds = match parts[0].trim().parse::<u64>() {
                Ok(s) => s,
                Err(_) => return Err("Invalid seconds format"),
            };
        },
        // Minutes:Seconds
        2 => {
            minutes = match parts[0].trim().parse::<u64>() {
                Ok(m) => m,
                Err(_) => return Err("Invalid minutes format"),
            };
            
            seconds = match parts[1].trim().parse::<u64>() {
                Ok(s) => s,
                Err(_) => return Err("Invalid seconds format"),
            };
        },
        // Hours:Minutes:Seconds
        3 => {
            hours = match parts[0].trim().parse::<u64>() {
                Ok(h) => h,
                Err(_) => return Err("Invalid hours format"),
            };
            
            minutes = match parts[1].trim().parse::<u64>() {
                Ok(m) => m,
                Err(_) => return Err("Invalid minutes format"),
            };
            
            seconds = match parts[2].trim().parse::<u64>() {
                Ok(s) => s,
                Err(_) => return Err("Invalid seconds format"),
            };
        },
        _ => return Err("Invalid time format. Use HH:MM:SS, MM:SS, or SS"),
    }
    
    // Handle overflow
    if seconds >= 60 {
        minutes += seconds / 60;
        seconds %= 60;
    }
    
    if minutes >= 60 {
        hours += minutes / 60;
        minutes %= 60;
    }
    
    // Convert to total seconds
    let total_seconds = hours * 3600 + minutes * 60 + seconds;
    Ok(total_seconds)
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    
    // Load configuration
    let config = Config::load()?;
    println!("Loaded configuration:");
    println!("  blink_separator = {}", config.blink_separator);
    println!("  countdown_color = {}", config.colors.countdown);
    println!("  stopwatch_color = {}", config.colors.stopwatch);
    println!("  countdown_refresh_rate = {}ms", config.countdown_refresh_rate);
    
    // Handle --init-config flag
    if cli.init_config {
        println!("Configuration file initialized.");
        return Ok(());
    }

    // Handle pomodoro mode
    if cli.pomodoro {
        println!("Starting Pomodoro timer (25min work, 5min break)");
        return run_pomodoro(&config);
    }

    // Handle countdown
    if let Some(time_str) = cli.countdown {
        match parse_time_string(&time_str) {
            Ok(total_seconds) => {
                if total_seconds == 0 {
                    println!("Please specify a valid countdown time greater than zero.");
                    return Ok(());
                }
                return run_countdown(total_seconds, &config);
            },
            Err(e) => {
                println!("Error parsing time: {}. Use format HH:MM:SS, MM:SS, or SS.", e);
                return Ok(());
            }
        }
    }
    
    // Handle stopwatch
    if cli.stopwatch {
        return run_stopwatch(&config);
    }
    
    // If no valid options provided, show usage
    println!("No valid command specified. Use -c/--countdown TIME, -s/--stopwatch, or -p/--pomodoro");
    Ok(())
}

// Helper function to reduce screen flicker by only updating what changed
fn stable_display(
    stdout: &mut io::Stdout, 
    ascii_time: &[String], 
    last_display: &mut Option<Vec<String>>,
    x_pos: u16,
    y_pos: u16,
    color: Color,
) -> io::Result<()> {
    // If this is the first time or the display size has changed
    if last_display.is_none() || last_display.as_ref().unwrap().len() != ascii_time.len() {
        // Display entire ascii art
        for (i, line) in ascii_time.iter().enumerate() {
            stdout.execute(cursor::MoveTo(x_pos, y_pos + i as u16))?;
            stdout.execute(style::PrintStyledContent(
                line.to_string().with(color)
            ))?;
        }
        *last_display = Some(ascii_time.to_vec());
        return Ok(());
    }
    
    // Only update lines that have changed
    let prev_display = last_display.as_ref().unwrap();
    for (i, (new_line, old_line)) in ascii_time.iter().zip(prev_display.iter()).enumerate() {
        if new_line != old_line {
            stdout.execute(cursor::MoveTo(x_pos, y_pos + i as u16))?;
            // Clear the old line first
            stdout.execute(Clear(ClearType::CurrentLine))?;
            stdout.execute(cursor::MoveTo(x_pos, y_pos + i as u16))?;
            stdout.execute(style::PrintStyledContent(
                new_line.to_string().with(color)
            ))?;
        }
    }
    
    // Update the saved display
    *last_display = Some(ascii_time.to_vec());
    Ok(())
}

fn run_countdown(total_seconds: u64, config: &Config) -> io::Result<()> {
    let mut stdout = stdout();
    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(total_seconds);
    
    // For tracking display changes
    let mut last_display: Option<Vec<String>> = None;

    // Setup terminal
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    // Clear screen once at the beginning
    stdout.execute(Clear(ClearType::All))?;
    
    // Display instructions (only once)
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(style::PrintStyledContent(
        "Press q or Ctrl+C to exit".with(config.ui_text_color())
    ))?;
    
    // Main timer loop
    loop {
        // Check for exit key (q or Ctrl+C)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                if code == KeyCode::Char('q') || 
                   (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
                    break;
                }
            }
        }
        
        let now = Instant::now();
        if now >= end_time {
            // Timer complete
            show_time_up(&mut stdout, config)?;
            break;
        }
        
        let remaining = end_time - now;
        let remaining_secs = remaining.as_secs();
        let minutes = remaining_secs / 60;
        let seconds = remaining_secs % 60;
        
        // Format time based on the original length
        let display_time = if minutes >= 60 {
            let hours = minutes / 60;
            let mins = minutes % 60;
            format!("{}:{:02}:{:02}", hours, mins, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        };
        
        // If blinking is enabled, alternate the colon visibility
        let display_with_blink = if config.blink_separator {
            // Toggle blink state about once per second
            // Use the time since start for consistent blinking
            let blink_on = (now.duration_since(start_time).as_millis() / 500) % 2 == 0;
            
            if blink_on {
                display_time
            } else {
                // Replace colons with spaces when blinked off
                display_time.replace(':', " ")
            }
        } else {
            display_time
        };
        
        // Get ASCII art representation
        let ascii_time = digit::render_time(&display_with_blink);
        
        // Display ASCII art time centered on screen
        let (term_width, term_height) = terminal::size()?;
        let time_width = ascii_time[0].len() as u16;
        let time_height = ascii_time.len() as u16;
        
        let x_pos = (term_width - time_width) / 2;
        let y_pos = (term_height - time_height) / 2;
        
        // Use our stable display function to avoid flickering
        stable_display(&mut stdout, &ascii_time, &mut last_display, x_pos, y_pos, config.countdown_color())?;
        
        stdout.flush()?;
        thread::sleep(Duration::from_millis(config.countdown_refresh_rate));
    }

    // Cleanup
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    println!("Timer complete!");
    Ok(())
}

fn show_time_up(stdout: &mut io::Stdout, config: &Config) -> io::Result<()> {
    let time_up_text = vec![
        "┌┬┐┬┌┬┐┌─┐ ┬┌─┐  ┬ ┬┌─┐┬",
        " │ ││││├┤  │└─┐  │ │├─┘│",
        " ┴ ┴┴ ┴└─┘ ┴└─┘  └─┘┴  o",
    ];
    
    // Get terminal size
    let (term_width, term_height) = terminal::size()?;
    
    // Calculate the width of the text (accounting for possible unicode width issues)
    // Using a fixed width for each string to ensure proper centering
    let text_width = 27u16; // Adjust this value if needed to match the actual width
    let text_height = time_up_text.len() as u16;
    
    // Calculate the position to center the text
    let x_pos = (term_width.saturating_sub(text_width)) / 2;
    let y_pos = (term_height.saturating_sub(text_height)) / 2;
    
    // Flash "TIME'S UP!" a few times
    for i in 0..5 {
        stdout.execute(Clear(ClearType::All))?;
        
        // Always display instructions at the top
        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(style::PrintStyledContent(
            "Press q or Ctrl+C to exit".with(config.ui_text_color())
        ))?;
        
        // Only display TIME'S UP on even iterations (creates flashing effect)
        if i % 2 == 0 {
            for (j, line) in time_up_text.iter().enumerate() {
                // Center each line individually to ensure perfect alignment
                stdout.execute(cursor::MoveTo(x_pos, y_pos + j as u16))?;
                stdout.execute(style::PrintStyledContent(
                    line.to_string().with(config.times_up_color()).bold()
                ))?;
            }
        }
        
        stdout.flush()?;
        
        // Check for exit key during the flashing animation
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                    if code == KeyCode::Char('q') || 
                       (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
                        return Ok(());
                    }
                }
            }
        }
    }
    
    // After flashing, keep showing the "TIME'S UP!" message until user exits
    stdout.execute(Clear(ClearType::All))?;
    
    // Display instructions at the top
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(style::PrintStyledContent(
        "Press q or Ctrl+C to exit".with(config.ui_text_color())
    ))?;
    
    // Display final "TIME'S UP!" message
    for (j, line) in time_up_text.iter().enumerate() {
        stdout.execute(cursor::MoveTo(x_pos, y_pos + j as u16))?;
        stdout.execute(style::PrintStyledContent(
            line.to_string().with(config.times_up_color()).bold()
        ))?;
    }
    
    stdout.flush()?;
    
    // Wait for user to exit
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                if code == KeyCode::Char('q') || 
                   (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
                    break;
                }
            }
        }
    }
    
    Ok(())
}

/// Run the Pomodoro timer with default settings (25min work, 5min break)
fn run_pomodoro(config: &Config) -> io::Result<()> {
    let mut stdout = stdout();
    let mut cycle = 1;
    let work_time = 25 * 60; // 25 minutes in seconds
    let break_time = 5 * 60; // 5 minutes in seconds
    
    // Setup terminal
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    // Clear screen once at the beginning
    stdout.execute(Clear(ClearType::All))?;
    
    loop {
        // Work session
        let session_name = format!("Work Session #{}", cycle);
        if !run_pomodoro_session(&mut stdout, &session_name, work_time, config.countdown_color(), config)? {
            break; // User quit
        }
        
        // Show a message that it's break time
        display_phase_change(&mut stdout, "Break Time!", config)?;
        
        // Break session
        let session_name = format!("Break #{}", cycle);
        if !run_pomodoro_session(&mut stdout, &session_name, break_time, config.stopwatch_color(), config)? {
            break; // User quit
        }
        
        // Show a message that it's work time again
        display_phase_change(&mut stdout, "Back to Work!", config)?;
        
        // Increment cycle counter
        cycle += 1;
    }
    
    // Cleanup
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    println!("Pomodoro timer ended. Completed {} full cycles.", cycle - 1);
    Ok(())
}

/// Display a phase change message between Pomodoro sessions
/// Returns true if user wants to continue, false if they want to quit
fn display_phase_change(stdout: &mut io::Stdout, message: &str, config: &Config) -> io::Result<bool> {
    stdout.execute(Clear(ClearType::All))?;
    
    // Get terminal size
    let (term_width, term_height) = terminal::size()?;
    
    // Display instructions at the top
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(style::PrintStyledContent(
        "Press q or Ctrl+C to exit, any other key to continue".with(config.ui_text_color())
    ))?;
    
    // Display the phase change message centered
    let msg_x = (term_width as usize).saturating_sub(message.len()) / 2;
    let msg_y = term_height / 2;
    
    stdout.execute(cursor::MoveTo(msg_x as u16, msg_y))?;
    stdout.execute(style::PrintStyledContent(
        message.to_string().with(config.times_up_color()).bold()
    ))?;
    
    stdout.flush()?;
    
    // Wait for user input to continue or quit
    if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
        if code == KeyCode::Char('q') || 
           (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Run a single session of the Pomodoro timer (either work or break)
/// Returns true if the session completed normally, false if user quit
fn run_pomodoro_session(
    stdout: &mut io::Stdout, 
    session_name: &str, 
    duration_secs: u64, 
    color: Color, 
    config: &Config
) -> io::Result<bool> {
    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);
    
    // For tracking display changes
    let mut last_display: Option<Vec<String>> = None;

    // Clear screen
    stdout.execute(Clear(ClearType::All))?;
    
    // Display instructions and session info
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(style::PrintStyledContent(
        "Press q or Ctrl+C to exit".with(config.ui_text_color())
    ))?;
    
    stdout.execute(cursor::MoveTo(0, 1))?;
    stdout.execute(style::PrintStyledContent(
        format!("Current: {}", session_name).with(config.ui_text_color())
    ))?;
    
    // Main timer loop
    loop {
        // Check for exit key (q or Ctrl+C)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                if code == KeyCode::Char('q') || 
                   (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
                    return Ok(false); // User quit
                }
            }
        }
        
        let now = Instant::now();
        if now >= end_time {
            // Session complete
            show_session_complete(stdout, session_name, config)?;
            return Ok(true); // Session completed normally
        }
        
        let remaining = end_time - now;
        let remaining_secs = remaining.as_secs();
        let minutes = remaining_secs / 60;
        let seconds = remaining_secs % 60;
        
        // Format time
        let display_time = format!("{}:{:02}", minutes, seconds);
        
        // Apply blinking effect if enabled
        let display_with_blink = if config.blink_separator {
            let blink_on = (now.duration_since(start_time).as_millis() / 500) % 2 == 0;
            if blink_on { display_time } else { display_time.replace(':', " ") }
        } else {
            display_time
        };
        
        // Get ASCII art representation
        let ascii_time = digit::render_time(&display_with_blink);
        
        // Display ASCII art time centered on screen
        let (term_width, term_height) = terminal::size()?;
        let time_width = ascii_time[0].len() as u16;
        let time_height = ascii_time.len() as u16;
        
        let x_pos = (term_width - time_width) / 2;
        let y_pos = (term_height - time_height) / 2;
        
        // Use our stable display function to avoid flickering
        stable_display(stdout, &ascii_time, &mut last_display, x_pos, y_pos, color)?;
        
        stdout.flush()?;
        thread::sleep(Duration::from_millis(config.countdown_refresh_rate));
    }
}

/// Show a session complete message
fn show_session_complete(stdout: &mut io::Stdout, session_name: &str, config: &Config) -> io::Result<()> {
    stdout.execute(Clear(ClearType::All))?;
    
    // Get terminal size
    let (term_width, term_height) = terminal::size()?;
    
    // Display instructions at the top
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(style::PrintStyledContent(
        "Press any key to continue".with(config.ui_text_color())
    ))?;
    
    // Display session complete message
    let message = format!("{} Complete!", session_name);
    let msg_x = (term_width as usize).saturating_sub(message.len()) / 2;
    let msg_y = term_height / 2;
    
    stdout.execute(cursor::MoveTo(msg_x as u16, msg_y))?;
    stdout.execute(style::PrintStyledContent(
        message.with(config.times_up_color()).bold()
    ))?;
    
    stdout.flush()?;
    
    // Wait for any key press
    event::read()?;
    
    Ok(())
}

fn run_stopwatch(config: &Config) -> io::Result<()> {
    let mut stdout = stdout();
    let start_time = Instant::now();
    
    // For tracking display changes
    let mut last_display: Option<Vec<String>> = None;

    // Setup terminal
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    
    // Clear screen once at the beginning
    stdout.execute(Clear(ClearType::All))?;
    
    // Display instructions (only once)
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(style::PrintStyledContent(
        "Press q or Ctrl+C to exit".with(config.ui_text_color())
    ))?;

    // Main stopwatch loop
    loop {
        // Check for exit key (q or Ctrl+C)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                if code == KeyCode::Char('q') || 
                   (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
                    break;
                }
            }
        }
        
        let now = Instant::now();
        let elapsed = now - start_time;
        let elapsed_secs = elapsed.as_secs();
        let minutes = elapsed_secs / 60;
        let seconds = elapsed_secs % 60;
        let centisecs = elapsed.subsec_millis() / 10;
        
        // Format time
        let display_time = format!("{}:{:02}.{:02}", minutes, seconds, centisecs);
        
        // If blinking is enabled, alternate the colon visibility
        let display_with_blink = if config.blink_separator {
            // Toggle blink state about once per second
            let blink_on = (elapsed.as_millis() / 500) % 2 == 0;
            
            if blink_on {
                display_time
            } else {
                // Replace colons with spaces when blinked off
                display_time.replace(':', " ")
            }
        } else {
            display_time
        };
        
        // Get ASCII art representation
        let ascii_time = digit::render_time(&display_with_blink);
        
        // Display ASCII art time centered on screen
        let (term_width, term_height) = terminal::size()?;
        let time_width = ascii_time[0].len() as u16;
        let time_height = ascii_time.len() as u16;
        
        let x_pos = (term_width - time_width) / 2;
        let y_pos = (term_height - time_height) / 2;
        
        // Use our stable display function
        stable_display(&mut stdout, &ascii_time, &mut last_display, x_pos, y_pos, config.stopwatch_color())?;
        
        stdout.flush()?;
        thread::sleep(Duration::from_millis(config.stopwatch_refresh_rate));
    }

    // Cleanup
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    println!("Stopwatch stopped!");
    Ok(())
}