use std::collections::HashMap;
use std::io::{self, Write};

use crate::{Dependency, DependencyComparisonResult};

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Gray,
    Cyan,
    Red,
    Yellow,
    Green,
}

pub struct Console {
    dry_run: bool,
}

impl Console {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    pub fn write(&self, text: &str, color: Color) {
        self.write_padded(text, color, 0);
    }

    pub fn write_padded(&self, text: &str, color: Color, column_width: usize) {
        let color = self.effective_color(color);
        let output = if column_width > 0 {
            format!("{text:<column_width$}")
        } else {
            text.to_string()
        };

        if is_ansi_enabled() {
            let _ = write!(io::stdout(), "{}{output}{ANSI_RESET}", to_ansi(color));
            return;
        }

        let _ = write!(io::stdout(), "{output}");
    }

    pub fn write_line(&self, text: &str, color: Color) {
        let color = self.effective_color(color);
        if is_ansi_enabled() {
            println!("{}{text}{ANSI_RESET}", to_ansi(color));
            return;
        }

        println!("{text}");
    }

    pub fn show_group_result(&self, target_framework: &str, result: &DependencyComparisonResult) {
        self.write_line(
            &format!("<group targetFramework=\"{target_framework}\">"),
            Color::Cyan,
        );
        self.show_result(result);
    }

    pub fn show_result(&self, result: &DependencyComparisonResult) {
        let column_width_helper_list: Vec<_> = result
            .updated_references
            .iter()
            .chain(&result.no_changes_references)
            .chain(&result.added_references)
            .chain(&result.deleted_references)
            .collect();

        if column_width_helper_list.is_empty() {
            self.write_line("\t (no dependency changes)", Color::Gray);
            println!();
            return;
        }

        let column_width = determine_column_name_width(&column_width_helper_list) + 5;

        if !result.deleted_references.is_empty() {
            self.write_line(
                &format!("\t Deleted references {}:", result.deleted_references.len()),
                Color::Red,
            );
            for item in &result.deleted_references {
                self.write("\t\t Name:", Color::Gray);
                self.write_padded(&format!(" {}", item.name), Color::Red, column_width);
                self.write("Version: ", Color::Gray);
                self.write(&item.version, Color::Red);
                println!();
            }
            println!();
        }

        if !result.updated_references.is_empty() {
            let column_version_width =
                determine_column_version_width(&result.outdated_references) + 5;
            self.write_line(
                &format!("\t Updated references {}:", result.updated_references.len()),
                Color::Yellow,
            );
            for item in &result.updated_references {
                self.write("\t\t Name:", Color::Gray);
                self.write_padded(&format!(" {}", item.name), Color::Yellow, column_width);
                self.write("Version: ", Color::Gray);
                if let Some(old_version) = result.outdated_references.get(&item.name) {
                    self.write_padded(&format!("{old_version} "), Color::Gray, column_version_width);
                }
                self.write(&format!("-> {}", item.version), Color::Yellow);
                println!();
            }
            println!();
        }

        if !result.added_references.is_empty() {
            self.write_line(
                &format!("\t Added references {}:", result.added_references.len()),
                Color::Green,
            );
            for item in &result.added_references {
                self.write("\t\t Name:", Color::Gray);
                self.write_padded(&format!(" {}", item.name), Color::Green, column_width);
                self.write("Version: ", Color::Gray);
                self.write(&item.version, Color::Green);
                println!();
            }
            println!();
        }

        if !result.no_changes_references.is_empty() {
            self.write_line(
                &format!(
                    "\t Not changed references {}:",
                    result.no_changes_references.len()
                ),
                Color::Gray,
            );
            for item in &result.no_changes_references {
                self.write("\t\t Name:", Color::Gray);
                self.write_padded(&format!(" {}", item.name), Color::Gray, column_width);
                self.write("Version: ", Color::Gray);
                self.write(&item.version, Color::Gray);
                println!();
            }
            println!();
        }
    }

    fn effective_color(&self, color: Color) -> Color {
        if self.dry_run {
            Color::Gray
        } else {
            color
        }
    }
}

const ANSI_RESET: &str = "\u{001b}[0m";

fn is_ansi_enabled() -> bool {
    std::env::var("CONSOLE_ANSI_COLOR").is_ok_and(|value| {
        value.eq_ignore_ascii_case("true")
            || value == "1"
            || value.eq_ignore_ascii_case("yes")
    })
}

fn to_ansi(color: Color) -> &'static str {
    match color {
        Color::Gray => "\u{001b}[90m",
        Color::Cyan => "\u{001b}[96m",
        Color::Red => "\u{001b}[91m",
        Color::Yellow => "\u{001b}[93m",
        Color::Green => "\u{001b}[92m",
    }
}

fn determine_column_name_width(references: &[&Dependency]) -> usize {
    references
        .iter()
        .map(|dependency| dependency.name.len())
        .max()
        .unwrap_or(0)
}

fn determine_column_version_width(outdated_references: &HashMap<String, String>) -> usize {
    outdated_references
        .values()
        .map(String::len)
        .max()
        .unwrap_or(0)
}
