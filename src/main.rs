use clap::{ArgEnum, Parser};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum ErrorMode {
    Panic,
    Skip,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    filename: String,

    #[clap(short = 'F', long)]
    folders: String,

    #[clap(short, long)]
    column: String,

    #[clap(short, long, default_value = "./")]
    root: String,

    #[clap(short, long, default_value = "output.csv")]
    output: String,

    #[clap(short, long)]
    verbose: bool,

    #[clap(short, long, arg_enum, default_value = "panic")]
    error: ErrorMode,
}

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

fn open_input(path: &PathBuf) -> Option<BufReader<File>> {
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
    match file {
        Some(file) => Some(BufReader::new(file)),
        None => None,
    }
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

        // Read the header, but only include if on first iteration (to avoid dupes)
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
