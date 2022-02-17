use clap::{ArgEnum, Parser};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, ErrorKind};
use std::path::Path;

/// Different error modes that control the program's behaviour when an input
/// file is not found in one of the provided folders.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum ErrorMode {
    /// The program should panic (and abort) if an input file is not found.
    Panic,
    /// The program should silently ignore and skip missing input files.
    Skip,
}

/// The program's CLI arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The name of the input file to look for in each folder.
    #[clap(short, long)]
    filename: String,

    /// The root directory of the input folders.
    ///
    /// By default, the root directory is the current working directory.
    #[clap(short, long, default_value = "./")]
    root: String,

    /// A comma-delimited list of folders to look in for the input file.
    #[clap(short = 'F', long)]
    folders: String,

    /// The name of the column that is added to the output CSV file, containing
    /// the name of the folder that each row originated from.
    #[clap(short, long)]
    column: String,

    /// The name of the output CSV file.
    ///
    /// By default, the output file is `output.csv`.
    #[clap(short, long, default_value = "output.csv")]
    output: String,

    /// Should additional/debugging messages be logged?
    #[clap(short, long)]
    verbose: bool,

    /// Controls the behaviour of the program when an file is not found.
    ///
    /// By default, the behaviour is to panic (and abort).
    #[clap(short, long, arg_enum, default_value = "panic")]
    error: ErrorMode,
}

/// Creates the output file that will contain the aggregated CSV data.
///
/// This function first attempts to create a file at the provided path: if the
/// file already exists, it is truncated.
///
/// After this, a BufWriter is initialized for the newly created/truncated file.
///
/// # Panics
///
/// The function will panic if the program doesn't have write permissions for
/// the provided file path, or if any other generic error is encountered during
/// the file creation.
///
/// # Examples
///
/// ```
/// let mut out = create_output(path);
/// writeln!(out, "Hello world!");
/// ```
fn create_output(path: String) -> BufWriter<File> {
    let file = match File::create(path) {
        Ok(file) => file,
        Err(err) => match err.kind() {
            ErrorKind::PermissionDenied => {
                panic!("Permission denied when trying to create output file!")
            }
            other => panic!(
                "Encountered an error when creating output file: {:?}",
                other
            ),
        },
    };
    BufWriter::new(file)
}

/// Attempts to open an input CSV file (that will be aggregated into the output
/// file).
///
/// This function attempts to open the file at the provided path, and then
/// initializes a BufReader.
///
/// The function returns an option which resolves to `None` if the file was not
/// found.
///
/// # Panics
///
/// The function will panic if the program doesn't have read permissions for
/// the provided file path, or if any other generic error is encountered during
/// the read operation on the file.
///
/// # Examples
///
/// ```
/// let mut reader = match open_input(&path) {
///     Some(file) => file,
///     None => panic!("File not found!"),
/// };
/// ```
fn open_input(path: &Path) -> Option<BufReader<File>> {
    let folder = path.parent().unwrap().as_os_str().to_string_lossy();
    let file = match File::open(path) {
        Ok(file) => Some(file),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => None,
            ErrorKind::PermissionDenied => panic!(
                "Permission denied when trying to read file in folder '{}'!",
                folder
            ),
            other => panic!(
                "Encountered an error when reading file in folder '{}': {:?}",
                folder, other
            ),
        },
    };
    file.map(BufReader::new)
}

fn main() {
    let args = Args::parse();
    let mut out = create_output(args.output);
    let mut lines = Vec::new();
    let mut found_header = false;

    // Expect folder names to be comma-delimited
    for folder in args.folders.split(',') {
        let path = Path::new(&args.root).join(folder).join(&args.filename);
        let mut reader = match open_input(&path) {
            Some(reader) => reader,
            None => {
                // Skip files that don't exist if `--error=skip`
                if args.error == ErrorMode::Skip {
                    if args.verbose {
                        println!("Couldn't find file in folder '{}', so skipping...", folder);
                    }
                    continue;
                } else {
                    panic!("Couldn't find file in folder '{}'!", folder)
                }
            }
        };

        // Read the header, but only include if the header hasn't been found yet (to avoid dupes)
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .unwrap_or_else(|_| panic!("Failed to read header from file in folder '{}'!", folder));
        if !found_header {
            lines.extend_from_slice(
                vec![
                    args.column.clone(),
                    ",".to_owned(),
                    header.replace("\r\n", "\n"),
                ]
                .as_slice(),
            );
            found_header = true;
        }

        for line in reader.lines().filter_map(|result| result.ok()) {
            lines.extend_from_slice(
                vec![
                    folder.to_owned(),
                    String::from(","),
                    line,
                    String::from("\n"),
                ]
                .as_slice(),
            );
        }
    }

    out.write_all(lines.join("").as_bytes())
        .expect("Failed to write to output file!");
}
