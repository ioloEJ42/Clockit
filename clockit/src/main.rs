// src/main.rs
mod digit;

use clap::Parser;
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

    // Handle countdown
    if let Some(time_str) = cli.countdown {
        match parse_time_string(&time_str) {
            Ok(total_seconds) => {
                if total_seconds == 0 {
                    println!("Please specify a valid countdown time greater than zero.");
                    return Ok(());
                }
                return run_countdown(total_seconds);
            },
            Err(e) => {
                println!("Error parsing time: {}. Use format HH:MM:SS, MM:SS, or SS.", e);
                return Ok(());
            }
        }
    }
    
    // Handle stopwatch
    if cli.stopwatch {
        return run_stopwatch();
    }
    
    // If no valid options provided, show usage
    println!("No valid command specified. Use -c/--countdown TIME or -s/--stopwatch");
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

fn run_countdown(total_seconds: u64) -> io::Result<()> {
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
        "Press q or Ctrl+C to exit".with(Color::DarkGrey)
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
            show_time_up(&mut stdout)?;
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
        
        // Get ASCII art representation
        let ascii_time = digit::render_time(&display_time);
        
        // Display ASCII art time centered on screen
        let (term_width, term_height) = terminal::size()?;
        let time_width = ascii_time[0].len() as u16;
        let time_height = ascii_time.len() as u16;
        
        let x_pos = (term_width - time_width) / 2;
        let y_pos = (term_height - time_height) / 2;
        
        // Use our stable display function to avoid flickering
        stable_display(&mut stdout, &ascii_time, &mut last_display, x_pos, y_pos, Color::Cyan)?;
        
        stdout.flush()?;
        thread::sleep(Duration::from_millis(200));
    }

    // Cleanup
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    println!("Timer complete!");
    Ok(())
}

fn show_time_up(stdout: &mut io::Stdout) -> io::Result<()> {
    let time_up_text = vec![
        "┌┬┐┬┌┬┐┌─┐ ┬┌─┐  ┬ ┬┌─┐┬",
        " │ ││││├┤  │└─┐  │ │├─┘│",
        " ┴ ┴┴ ┴└─┘ ┴└─┘  └─┘┴  o",
    ];
    
    let (term_width, term_height) = terminal::size()?;
    let text_width = time_up_text[0].len() as u16;
    let text_height = time_up_text.len() as u16;
    
    let x_pos = (term_width - text_width) / 2;
    let y_pos = (term_height - text_height) / 2;
    
    // Flash "TIME'S UP!" a few times
    for i in 0..5 {
        stdout.execute(Clear(ClearType::All))?;
        
        // Only display on even iterations (creates flashing effect)
        if i % 2 == 0 {
            for (j, line) in time_up_text.iter().enumerate() {
                stdout.execute(cursor::MoveTo(x_pos, y_pos + j as u16))?;
                stdout.execute(style::PrintStyledContent(
                    line.to_string().with(Color::Red).bold()
                ))?;
            }
        }
        
        stdout.flush()?;
        thread::sleep(Duration::from_millis(500));
    }
    
    Ok(())
}

fn run_stopwatch() -> io::Result<()> {
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
        "Press q or Ctrl+C to exit".with(Color::DarkGrey)
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
        
        // Get ASCII art representation
        let ascii_time = digit::render_time(&display_time);
        
        // Display ASCII art time centered on screen
        let (term_width, term_height) = terminal::size()?;
        let time_width = ascii_time[0].len() as u16;
        let time_height = ascii_time.len() as u16;
        
        let x_pos = (term_width - time_width) / 2;
        let y_pos = (term_height - time_height) / 2;
        
        // Use our stable display function
        stable_display(&mut stdout, &ascii_time, &mut last_display, x_pos, y_pos, Color::Green)?;
        
        stdout.flush()?;
        thread::sleep(Duration::from_millis(100));
    }

    // Cleanup
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    println!("Stopwatch stopped!");
    Ok(())
}