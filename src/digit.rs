// src/digit.rs
//! Module for rendering ASCII digits

/// Returns ASCII art representation of a digit (0-9)
/// Using simple ASCII characters to ensure consistent width rendering
pub fn get_digit(digit: u8) -> Vec<&'static str> {
  match digit {
      0 => vec![
          ".---.",
          "|   |",
          "|   |",
          "|   |",
          "'---'",
      ],
      1 => vec![
          "  .  ",
          "  |  ",
          "  |  ",
          "  |  ",
          "  |  ",
      ],
      2 => vec![
          ".---.",
          "    |",
          ".---.",
          "|    ",
          "'---'",
      ],
      3 => vec![
          ".---.",
          "    |",
          ".---.",
          "    |",
          "'---'",
      ],
      4 => vec![
          "|   |",
          "|   |",
          "'---|",
          "    |",
          "    |",
      ],
      5 => vec![
          ".---.",
          "|    ",
          "'---.",
          "    |",
          "'---'",
      ],
      6 => vec![
          ".---.",
          "|    ",
          "|---.",
          "|   |",
          "'---'",
      ],
      7 => vec![
          ".---.",
          "    |",
          "    |",
          "    |",
          "    |",
      ],
      8 => vec![
          ".---.",
          "|   |",
          "|---.",
          "|   |",
          "'---'",
      ],
      9 => vec![
          ".---.",
          "|   |",
          "'---|",
          "    |",
          "'---'",
      ],
      _ => vec![
          "     ",
          "     ",
          "     ",
          "     ",
          "     ",
      ],
  }
}

/// Returns ASCII art representation of a colon
pub fn get_colon() -> Vec<&'static str> {
  vec![
      "     ",
      "  o  ",
      "     ",
      "  o  ",
      "     ",
  ]
}

/// Returns ASCII art representation of a dot
pub fn get_dot() -> Vec<&'static str> {
  vec![
      "     ",
      "     ",
      "     ",
      "     ",
      "  o  ",
  ]
}

/// Combines multiple digit ASCII arts horizontally into one string
pub fn combine_digits(digits: Vec<Vec<&str>>) -> Vec<String> {
  let height = if !digits.is_empty() { digits[0].len() } else { 0 };
  let mut result = vec![String::new(); height];
  
  for digit in digits {
      for (i, line) in digit.iter().enumerate() {
          result[i].push_str(line);
      }
  }
  
  result
}

/// Renders a time string (like "12:34" or "1:23.45") as ASCII art
pub fn render_time(time_string: &str) -> Vec<String> {
  let mut digit_arts = Vec::new();
  
  for c in time_string.chars() {
      match c {
          '0'..='9' => {
              let digit = c.to_digit(10).unwrap() as u8;
              digit_arts.push(get_digit(digit));
          },
          ':' => {
              digit_arts.push(get_colon());
          },
          '.' => {
              digit_arts.push(get_dot());
          },
          _ => {
              // For any other character (space, etc.) just add empty space
              digit_arts.push(vec![
                  "     ",
                  "     ",
                  "     ",
                  "     ",
                  "     ",
              ]);
          }
      }
  }
  
  combine_digits(digit_arts)
}