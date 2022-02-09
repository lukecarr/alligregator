use clap::Parser;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, BufWriter};
use std::path::Path;

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
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let out = File::create(args.output.clone())?;
    let mut out = BufWriter::new(out);

    for (i, folder) in args.folders.split(',').enumerate() {
        let path = Path::new(&args.root).join(folder).join(&args.filename);

        if !path.exists() {
            println!(
                "'{}' not found in folder '{}', so skipping...",
                args.filename, folder
            );
            continue;
        }
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut header = String::new();
        reader.read_line(&mut header)?;
        if i == 0 {
            out.write_all(format!("{},{}", args.column, header).as_bytes())?;
        }

        for line in reader.lines().filter_map(|result| result.ok()) {
            writeln!(out, "{},{}", folder, line)?;
        }
        println!("{} written to {} successfully!", folder, args.output);
    }

    Ok(())
}
