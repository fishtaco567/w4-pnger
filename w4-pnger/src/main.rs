use clap::{arg, ArgAction, ArgGroup, ArgMatches, Command};

mod analyze;
mod compress;
mod convert;
mod pngstream;
mod wasm4png;

use analyze::Analyzer;
use convert::{Converter, OutputType};

fn main() {
    let matches = cmd().get_matches();

    match matches.subcommand() {
        Some(("convert", submatches)) => {
            let path = get_path(&submatches);

            let compress: bool = *submatches.get_one("compress").expect("defaulted by clap");

            let (output_type, output_file) = if let Some(output_file) =
                submatches.get_one::<String>("raw")
            {
                ("raw", output_file.as_str())
            } else if let Some(output_file) = submatches.get_one::<String>("text") {
                ("text", output_file.as_str())
            } else {
                unreachable!("clap requires either --rs or --raw or --text are set with the appropriate parameter passed")
            };

            Converter::new(
                path,
                output_file,
                OutputType::from_str(output_type),
                compress,
            )
            .run();
        }
        Some(("analyze", submatches)) => {
            let path = get_path(&submatches);

            Analyzer::new(path).run();
        }

        _ => unreachable!("clap will exit the program if a valid subcommand is not reached"),
    }
}

fn cmd() -> Command {
    Command::new("w4-png")
        .version("0.1")
        .author("Maddie Jaksa")
        .about("Png compression and data generation tool for WASM-4")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("convert")
                .about("Converts a .png file for use with WASM-4")
                .arg(arg!(-c --compress "Compress these files?").action(ArgAction::SetTrue))
                .arg(arg!(--raw <FILE> "Generate a raw file with sprites"))
                .arg(arg!(--text <FILE> "Generate a text file with sprites"))
                .group(
                    ArgGroup::new("output")
                        .required(true)
                        .args(&["raw", "text"]),
                )
                .arg(arg!([PATH]).required(true)),
        )
        .subcommand(
            Command::new("analyze")
                .about("Analyzes a .png file and reports its compression statistics")
                .arg(arg!([PATH]).required(true)),
        )
}

fn get_path(matches: &ArgMatches) -> &str {
    matches
        .get_one::<String>("PATH")
        .expect("clap requires this argument to be present")
        .as_str()
}

#[test]
fn verify_cmd() {
    cmd().debug_assert();
}
