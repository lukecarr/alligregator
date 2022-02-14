use clap::{ArgEnum, Parser};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, ErrorKind};
use std::path::Path;

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

fn main() {
    let args = Args::parse();

    // Create output file that all inputs will be aggregated into
    let out = match File::create(args.output.clone()) {
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

    let mut out = BufWriter::new(out);

    // Expect folder names to be comma-delimited
    for (i, folder) in args.folders.split(',').enumerate() {
        let path = Path::new(&args.root).join(folder).join(&args.filename);

        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
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
        let mut reader = BufReader::new(file);

        // Read the header, but only write if on first iteration (to avoid dupes)
        let mut header = String::new();
        reader.read_line(&mut header).expect(format!(
            "Failed to read header from file in folder '{}'!",
            folder
        ));
        if i == 0 {
            out.write_all(format!("{},{}", args.column, header).as_bytes())
                .expect("Failed to write header to output file!");
        }

        for line in reader.lines().filter_map(|result| result.ok()) {
            writeln!(out, "{},{}", folder, line).expect(format!(
                "Failed to write rows from folder '{}' to output file!",
                folder
            ));
        }

        if args.verbose {
            println!("{} written to {} successfully!", folder, args.output);
        }
    }
}
