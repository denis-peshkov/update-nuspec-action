use std::path::{Path, PathBuf};
use std::time::Instant;

use update_nuspec::{process_nuspec, LibError, ProcessStatus};
use walkdir::WalkDir;

use crate::cli::args::CliRunOptions;
use crate::cli::console::{Color, Console};
use crate::cli::package_json;

pub fn run(options: &CliRunOptions) -> Result<(), LibError> {
    let console = Console::new(options.dry_run);

    if options.dry_run {
        console.write_line("[DRY RUN] Files will not be modified.", Color::Yellow);
    }

    if !options.path.exists() {
        console.write_line(
            &format!("Path '{}' is not valid!", options.path.display()),
            Color::Red,
        );
        return Ok(());
    }

    let nuspec_count = update_nuspec_files(&options.path, options.dry_run, &console)?;

    if let Some(package_version) = &options.package_version {
        update_package_json_files(
            &options.path,
            package_version,
            &options.dependency_scope,
            options.dry_run,
            &console,
        )?;
    } else if nuspec_count == 0 {
        console.write_line("*.nuspec files not found!", Color::Red);
    }

    if options.dry_run {
        console.write_line("[DRY RUN] Completed without writing changes.", Color::Yellow);
    }

    Ok(())
}

fn update_nuspec_files(path: &Path, dry_run: bool, console: &Console) -> Result<usize, LibError> {
    let nuspec_files = collect_nuspec_files(path);

    for nuspec in &nuspec_files {
        let nuspec_directory = nuspec
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf());

        console.write("Start processing file: ", Color::Gray);
        console.write(&format!("{} \n", nuspec.display()), Color::Cyan);

        let stopwatch = Instant::now();
        match process_nuspec(nuspec, &nuspec_directory, dry_run)? {
            result if result.status == ProcessStatus::ProjectNameNotFound => {
                console.write_line(
                    &format!("ProjectName not found in: {}", nuspec.display()),
                    Color::Red,
                );
            }
            result if result.status == ProcessStatus::ProjectFileNotFound => {
                let project_name = result
                    .project_id
                    .as_deref()
                    .unwrap_or("project");
                console.write_line(
                    &format!(
                        "ProjectFile: {project_name} not found in: {}",
                        nuspec_directory.display()
                    ),
                    Color::Red,
                );
            }
            result => {
                if let Some(comparison) = &result.comparison {
                    console.show_result(comparison);
                }

                for group in &result.group_comparisons {
                    console.show_group_result(&group.target_framework, &group.comparison);
                }

                if dry_run {
                    console.write_line("[DRY RUN] Skipped saving nuspec file.", Color::Yellow);
                }

                let elapsed = stopwatch.elapsed();
                println!(
                    "Elapsed: {:02}:{:02}:{:02}.{:02}",
                    elapsed.as_secs() / 3600,
                    (elapsed.as_secs() % 3600) / 60,
                    elapsed.as_secs() % 60,
                    elapsed.subsec_millis() / 10
                );
            }
        }

        console.write("End processing file: ", Color::Gray);
        console.write(&format!("{} \n \n", nuspec.display()), Color::Cyan);
    }

    Ok(nuspec_files.len())
}

fn update_package_json_files(
    path: &Path,
    package_version: &str,
    dependency_scope: &str,
    dry_run: bool,
    console: &Console,
) -> Result<(), LibError> {
    let package_json_files = collect_package_json_files(path);

    if package_json_files.is_empty() {
        console.write_line("package.json files not found!", Color::Red);
        return Ok(());
    }

    for package_json_path in package_json_files {
        console.write("Start processing file: ", Color::Gray);
        console.write(
            &format!("{} \n", package_json_path.display()),
            Color::Cyan,
        );
        package_json::process(
            &package_json_path,
            package_version,
            dependency_scope,
            dry_run,
            console,
        )?;
        console.write("End processing file: ", Color::Gray);
        console.write(
            &format!("{} \n \n", package_json_path.display()),
            Color::Cyan,
        );
    }

    Ok(())
}

fn collect_nuspec_files(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|file_path| {
            file_path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("nuspec"))
        })
        .collect()
}

fn collect_package_json_files(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let file_path = entry.into_path();
            if file_path
                .file_name()
                .is_some_and(|name| name == "package.json")
                && !is_under_node_modules(&file_path)
            {
                Some(file_path)
            } else {
                None
            }
        })
        .collect()
}

fn is_under_node_modules(file_path: &Path) -> bool {
    file_path
        .components()
        .any(|component| component.as_os_str().eq_ignore_ascii_case("node_modules"))
}
