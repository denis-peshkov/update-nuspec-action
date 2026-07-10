use std::collections::HashMap;

use update_nuspec::cli::console::{Color, Console};
use update_nuspec::{Dependency, DependencyComparisonResult};

fn comparison_with_all_categories() -> DependencyComparisonResult {
    DependencyComparisonResult {
        deleted_references: vec![Dependency::new("Removed.Package", "1.0.0")],
        updated_references: vec![Dependency::new("Updated.Package", "2.0.0")],
        added_references: vec![Dependency::new("Added.Package", "3.0.0")],
        no_changes_references: vec![Dependency::new("Stable.Package", "4.0.0")],
        outdated_references: HashMap::from([(
            "Updated.Package".to_string(),
            "1.5.0".to_string(),
        )]),
    }
}

#[test]
fn show_result_renders_all_change_categories_without_ansi() {
    temp_env::with_var("CONSOLE_ANSI_COLOR", None::<&str>, || {
        let console = Console::new(false);
        console.show_result(&comparison_with_all_categories());
    });
}

#[test]
fn show_result_renders_all_change_categories_with_ansi() {
    temp_env::with_var("CONSOLE_ANSI_COLOR", Some("true"), || {
        let console = Console::new(false);
        console.show_result(&comparison_with_all_categories());
        console.write("plain", Color::Green);
        console.write_padded(" padded ", Color::Yellow, 12);
        console.write_line("line", Color::Cyan);
    });
}

#[test]
fn show_result_reports_no_dependency_changes_when_empty() {
    temp_env::with_var("CONSOLE_ANSI_COLOR", Some("1"), || {
        let console = Console::new(false);
        console.show_result(&DependencyComparisonResult::default());
    });
}

#[test]
fn show_group_result_renders_target_framework_header() {
    let console = Console::new(false);
    console.show_group_result("net8.0", &comparison_with_all_categories());
}

#[test]
fn dry_run_console_uses_gray_palette() {
    temp_env::with_var("CONSOLE_ANSI_COLOR", Some("yes"), || {
        let console = Console::new(true);
        console.write("dry-run", Color::Red);
        console.write_padded(" padded ", Color::Green, 8);
        console.write_line("line", Color::Yellow);
        console.show_result(&comparison_with_all_categories());
    });
}
