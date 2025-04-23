// src/config.rs
use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

/// Represents the color scheme for different timer elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColorScheme {
    /// Color for countdown timer display
    #[serde(default = "default_countdown_color")]
    pub countdown: String,
    
    /// Color for stopwatch display
    #[serde(default = "default_stopwatch_color")]
    pub stopwatch: String,
    
    /// Color for time's up message
    #[serde(default = "default_times_up_color")]
    pub times_up: String,
    
    /// Color for instructions and other UI text
    #[serde(default = "default_ui_text_color")]
    pub ui_text: String,
    
    /// Color for Pomodoro work sessions
    #[serde(default = "default_pomodoro_work_color")]
    pub pomodoro_work: String,
    
    /// Color for Pomodoro break sessions
    #[serde(default = "default_pomodoro_break_color")]
    pub pomodoro_break: String,
}

fn default_countdown_color() -> String {
    "cyan".to_string()
}

fn default_stopwatch_color() -> String {
    "green".to_string()
}

fn default_times_up_color() -> String {
    "red".to_string()
}

fn default_ui_text_color() -> String {
    "grey".to_string()
}

fn default_pomodoro_work_color() -> String {
    "red".to_string()
}

fn default_pomodoro_break_color() -> String {
    "green".to_string()
}

/// Represents Pomodoro timer settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PomodoroSettings {
    /// Default duration of work sessions in minutes
    #[serde(default = "default_pomodoro_work_duration")]
    pub work_duration: u64,
    
    /// Default duration of break sessions in minutes
    #[serde(default = "default_pomodoro_break_duration")]
    pub break_duration: u64,
    
    /// Default number of cycles (0 means infinite)
    #[serde(default = "default_pomodoro_cycles")]
    pub cycles: u64,
    
    /// Play sound when sessions end
    #[serde(default = "default_pomodoro_sound")]
    pub sound_enabled: bool,

    /// Refresh rate in milliseconds for the pomodoro timer
    #[serde(default = "default_pomodoro_refresh_rate")]
    pub refresh_rate: u64,
}

fn default_pomodoro_work_duration() -> u64 {
    25
}

fn default_pomodoro_break_duration() -> u64 {
    5
}

fn default_pomodoro_cycles() -> u64 {
    0 // 0 means infinite
}

fn default_pomodoro_sound() -> bool {
    false
}

fn default_pomodoro_refresh_rate() -> u64 {
    200
}

/// Configuration for the Clockit application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Color scheme for the application
    #[serde(default)]
    pub colors: ColorScheme,
    
    /// Whether to use a blinking effect for the time separator
    #[serde(default = "default_blink_separator")]
    pub blink_separator: bool,
    
    /// Refresh rate in milliseconds for the countdown timer
    #[serde(default = "default_countdown_refresh_rate")]
    pub countdown_refresh_rate: u64,
    
    /// Refresh rate in milliseconds for the stopwatch
    #[serde(default = "default_stopwatch_refresh_rate")]
    pub stopwatch_refresh_rate: u64,
    
    /// Pomodoro timer settings
    #[serde(default)]
    pub pomodoro: PomodoroSettings,
}

fn default_blink_separator() -> bool {
    false
}

fn default_countdown_refresh_rate() -> u64 {
    200
}

fn default_stopwatch_refresh_rate() -> u64 {
    100
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            countdown: default_countdown_color(),
            stopwatch: default_stopwatch_color(),
            times_up: default_times_up_color(),
            ui_text: default_ui_text_color(),
            pomodoro_work: default_pomodoro_work_color(),
            pomodoro_break: default_pomodoro_break_color(),
        }
    }
}

impl Default for PomodoroSettings {
    fn default() -> Self {
        PomodoroSettings {
            work_duration: default_pomodoro_work_duration(),
            break_duration: default_pomodoro_break_duration(),
            cycles: default_pomodoro_cycles(),
            sound_enabled: default_pomodoro_sound(),
            refresh_rate: default_pomodoro_refresh_rate(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            colors: ColorScheme::default(),
            blink_separator: default_blink_separator(),
            countdown_refresh_rate: default_countdown_refresh_rate(),
            stopwatch_refresh_rate: default_stopwatch_refresh_rate(),
            pomodoro: PomodoroSettings::default(),
        }
    }
}

impl Config {
    /// Load configuration from a file, or create a default one if not found
    pub fn load() -> io::Result<Self> {
        let config_path = get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::create_default_config()?);
        }
        
        match fs::read_to_string(&config_path) {
            Ok(contents) => {
                match serde_yaml::from_str(&contents) {
                    Ok(config) => Ok(config),
                    Err(e) => {
                        eprintln!("Error parsing config file: {}. Using defaults.", e);
                        Ok(Config::default())
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading config file: {}. Using defaults.", e);
                Ok(Config::default())
            }
        }
    }
    
    /// Create a default configuration file and return the default config
    fn create_default_config() -> io::Result<Self> {
        let config_path = get_config_path()?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let default_config = Config::default();
        let yaml = serde_yaml::to_string(&default_config)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Add helpful comments to the YAML file
        let commented_yaml = format!(
            "# Clockit Configuration File\n\
            #\n\
            # Available colors: black, blue, cyan, dark_blue, dark_cyan, dark_green,\n\
            # dark_grey, dark_green, dark_magenta, dark_red, dark_yellow, green, grey,\n\
            # magenta, red, white, yellow\n\
            #\n\
            # countdown_refresh_rate: Time in ms between updates for countdown timer\n\
            # stopwatch_refresh_rate: Time in ms between updates for stopwatch\n\
            # blink_separator: Whether to make the colon/separators blink\n\
            #\n\
            # Pomodoro settings:\n\
            # work_duration: Duration of work sessions in minutes\n\
            # break_duration: Duration of break sessions in minutes\n\
            # cycles: Number of cycles to run (0 means infinite)\n\
            # sound_enabled: Play sound when sessions end (not implemented yet)\n\
            # refresh_rate: Update frequency in milliseconds\n\
            \n{}", yaml);
        
        fs::write(&config_path, commented_yaml)?;
        println!("Created default configuration at: {:?}", config_path);
        
        Ok(default_config)
    }
    
    /// Get the crossterm Color enum from a string color name
    pub fn parse_color(&self, color_name: &str) -> Color {
        match color_name.to_lowercase().as_str() {
            "black" => Color::Black,
            "blue" => Color::Blue,
            "cyan" => Color::Cyan,
            "dark_blue" => Color::DarkBlue,
            "dark_cyan" => Color::DarkCyan,
            "dark_green" => Color::DarkGreen,
            "dark_grey" | "dark_gray" => Color::DarkGrey,
            "dark_magenta" => Color::DarkMagenta,
            "dark_red" => Color::DarkRed,
            "dark_yellow" => Color::DarkYellow,
            "green" => Color::Green,
            "grey" | "gray" => Color::Grey,
            "magenta" => Color::Magenta,
            "red" => Color::Red,
            "white" => Color::White,
            "yellow" => Color::Yellow,
            _ => {
                eprintln!("Unknown color: {}. Using default.", color_name);
                Color::Reset
            }
        }
    }
    
    /// Get countdown color
    pub fn countdown_color(&self) -> Color {
        self.parse_color(&self.colors.countdown)
    }
    
    /// Get stopwatch color
    pub fn stopwatch_color(&self) -> Color {
        self.parse_color(&self.colors.stopwatch)
    }
    
    /// Get times up color
    pub fn times_up_color(&self) -> Color {
        self.parse_color(&self.colors.times_up)
    }
    
    /// Get UI text color
    pub fn ui_text_color(&self) -> Color {
        self.parse_color(&self.colors.ui_text)
    }
    
    /// Get Pomodoro work session color
    pub fn pomodoro_work_color(&self) -> Color {
        self.parse_color(&self.colors.pomodoro_work)
    }
    
    /// Get Pomodoro break session color
    pub fn pomodoro_break_color(&self) -> Color {
        self.parse_color(&self.colors.pomodoro_break)
    }
}

/// Get the path to the configuration file
fn get_config_path() -> io::Result<PathBuf> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir.join("clockit"),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find config directory",
            ))
        }
    };
    
    Ok(config_dir.join("config.yaml"))
}