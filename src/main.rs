use csv::Reader;
use std::{io, process};
use translocate::{get_file_location, get_file_reader, run, CliArgs, Config};
use yansi::Paint;

const APP_DESC: &str = "trans·lo·cate, verb, to move from one place to another.";
const MISSING_FILE_ERR: &str = "Try again with the absolute (full) path to the file.";

fn main() -> Result<(), std::io::Error> {
    let cli: CliArgs = argh::from_env();

    if cli.version.is_some() {
        return Ok(println!(
            "{} v{}\n{}",
            env!("CARGO_PKG_NAME").underline(),
            env!("CARGO_PKG_VERSION"),
            APP_DESC.italic()
        ));
    }

    let file_path = &cli.file;

    let csv_path = get_file_location(file_path)?;

    let config = Config::new(&cli, csv_path.extension());

    let mut reader = match get_file_reader(file_path, &config) {
        Ok(res) => res,
        Err(err) => match err.kind() {
            csv::ErrorKind::Io(io_error) => {
                if io_error.kind() == io::ErrorKind::NotFound {
                    eprintln!(
                        "{} file `{}` not found. {}",
                        "Error:".bold().on_bright_red(),
                        file_path.bold(),
                        MISSING_FILE_ERR.underline()
                    );
                    process::exit(1)
                } else {
                    eprintln!("{} {}", "Error:".bold().on_bright_red(), io_error);
                    process::exit(1)
                }
            }
            _ => {
                eprintln!("{} {}", "Error:".bold().on_bright_red(), err);
                process::exit(1)
            }
        },
    };

    let mut reader_count = Reader::from_path(&csv_path)?;

    let headings = reader.headers()?.clone();
    let rows = reader_count.byte_records().count();

    if let Err(error) = run(&mut reader, &headings, rows, &config) {
        eprintln!("{}", error);
        process::exit(1)
    }

    Ok(println!("\n✨🎉✨ {}", "Conversion successful!".bold()))
}
