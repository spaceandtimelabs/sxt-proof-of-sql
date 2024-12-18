//! This binary applies a preprocessing step to Solidity files that allows for importing Yul code from other files.

use clap::Parser;
use snafu::Snafu;
use std::{
    fs::{self, File},
    io::{self, BufRead, BufWriter, Write},
    path::Path,
};

const IMPORT_YUL: &str = "// IMPORT-YUL";
const END_IMPORT_YUL: &str = "// END-IMPORT-YUL";
const START_YUL: &str = "// START-YUL";
const END_YUL: &str = "// END-YUL";
const IMPORTED_YUL: &str = "// IMPORTED-YUL";
const END_IMPORTED_YUL: &str = "// END-IMPORTED-YUL";

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(display("Ill-formed IMPORT-YUL statement at line {line}"))]
    IllFormedImportYul { line: usize },
    #[snafu(display("Unmatched END-IMPORT-YUL at line {line}"))]
    UnmatchedEndImportYul { line: usize },
    #[snafu(display("Unmatched IMPORT-YUL at line"))]
    UnmatchedImportYul,
    #[snafu(display("Function {function_name} not found in file {file_path}"))]
    FunctionNotFound {
        function_name: String,
        file_path: String,
    },
}

/// A preprocessor for Solidity files to import Yul code
///
/// This tool processes a given file or directory, replacing the import statements with the corresponding Yul code.
///
/// # Usage
///
/// The Yul code should be wrapped in `// START-YUL <function_name>` and `// END-YUL` comments in the source files.
/// The import statement should be in the form `// IMPORT-YUL <file_path>:<function_name>` in the target Solidity files.
///
/// # Example
///
/// Given a Solidity file `example.psol` with the following content:
///
/// ```solidity
/// // IMPORT-YUL yul_code.sol:my_function
/// // END-IMPORT-YUL
/// ```
///
/// And a Yul file `yul_code.sol` with the following content:
///
/// ```solidity
/// // START-YUL my_function
/// function my_function() -> result {
///     // Yul code here
/// }
/// // END-YUL
/// ```
///
/// Running the binary will produce an output file `example.p.sol` with the following content:
///
/// ```solidity
/// // IMPORTED-YUL yul_code.sol:my_function
/// function my_function() -> result {
///     // Yul code here
/// }
/// // END-IMPORTED-YUL
/// ```
#[derive(Parser, Debug)]
#[command(about, long_about)]
struct Args {
    /// The path to the file or directory to process
    path: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    process_path(Path::new(&args.path))?;
    Ok(())
}

fn process_path(path: &Path) -> Result<(), Error> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            process_path(&entry?.path())?;
        }
    } else if path.extension().and_then(|ext| ext.to_str()) == Some("psol") {
        process_file(path)?;
    }
    Ok(())
}

fn process_file(path: &Path) -> Result<(), Error> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut output_lines = Vec::new();
    let mut inside_import = false;
    let mut import_file = String::new();
    let mut function_name = String::new();
    let base_path = path.parent().unwrap_or_else(|| Path::new(""));

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if let Some(import_pos) = line.find(IMPORT_YUL) {
            let parts: Vec<&str> = line[import_pos + IMPORT_YUL.len()..].split(':').collect();
            if parts.len() != 2 {
                return Err(Error::IllFormedImportYul {
                    line: line_number + 1,
                });
            }
            inside_import = true;
            import_file = parts[0].trim().to_string();
            function_name = parts[1].trim().to_string();
        } else if line.contains(END_IMPORT_YUL) {
            if !inside_import {
                return Err(Error::UnmatchedEndImportYul {
                    line: line_number + 1,
                });
            }
            let function_lines = extract_function(&base_path.join(&import_file), &function_name)?;
            output_lines.push(format!("{IMPORTED_YUL} {import_file}:{function_name}"));
            output_lines.extend(function_lines);
            output_lines.push(END_IMPORTED_YUL.to_string());
            inside_import = false;
        } else if !inside_import {
            output_lines.push(line);
        }
    }

    if inside_import {
        return Err(Error::UnmatchedImportYul);
    }

    let file = File::create(path.with_extension("p.sol"))?;
    let mut writer = BufWriter::new(file);
    for line in output_lines {
        writeln!(writer, "{line}")?;
    }

    Ok(())
}

fn extract_function(file_path: &Path, function_name: &str) -> Result<Vec<String>, Error> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut function_lines = Vec::new();
    let mut inside_function = false;

    for line in reader.lines() {
        let line = line?;
        if line.contains(&format!("{START_YUL} {function_name}")) {
            inside_function = true;
        } else if line.contains(END_YUL) {
            break;
        } else if inside_function {
            function_lines.push(line);
        }
    }

    if !inside_function {
        return Err(Error::FunctionNotFound {
            function_name: function_name.to_string(),
            file_path: file_path.display().to_string(),
        });
    }

    Ok(function_lines)
}
