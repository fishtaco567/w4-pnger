use clap::{arg, AppSettings, ArgAction, ArgGroup, Command};

mod analyze;
mod convert;
mod pngstream;
mod wasm4png;

use analyze::Analyzer;
use convert::{Converter, OutputType};

fn main() {
    let matches = cmd().get_matches();

    match matches.subcommand() {
        Some(("convert", submatches)) => {
            let path = submatches
                .get_one::<String>("PATH")
                .expect("clap requires this argument to be present")
                .as_str();

            let compress: bool = *submatches.get_one("compress").expect("defaulted by clap");

            let (output_type, output_file) =
                if let Some(output_file) = submatches.get_one::<String>("rs") {
                    ("rust", output_file.as_str())
                } else if let Some(output_file) = submatches.get_one::<String>("raw") {
                    ("raw", output_file.as_str())
                } else if let Some(output_file) = submatches.get_one::<String>("text") {
                    ("text", output_file.as_str())
                } else {
                    //clap requires either --rs or --raw or --text are set with the appropriate parameter passed
                    unreachable!()
                };

            Converter::new(
                path,
                output_file,
                OutputType::from_str(output_type),
                compress,
            )
            .run();
        }
        Some(("analyze", submatches)) => todo!(),

        //clap will exit the program if a valid subcommand is not reached
        _ => unreachable!(),
    }
}

fn cmd() -> Command<'static> {
    Command::new("w4-png")
        .version("0.1")
        .author("Maddie Jaksa")
        .about("Png compression and data generation tool for WASM-4")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            Command::new("convert")
                .arg(arg!(-c --compress "Compress these files?").action(ArgAction::SetTrue))
                .arg(arg!(--rs <FILE> "Generate a rust file with sprites"))
                .arg(arg!(--raw <FILE> "Generate a raw file with sprites"))
                .arg(arg!(--text <FILE> "Generate a text file with sprites"))
                .group(
                    ArgGroup::new("output")
                        .required(true)
                        .args(&["rs", "raw", "text"]),
                )
                .arg(arg!([PATH]).required(true)),
        )
        .subcommand(Command::new("analyze").arg(arg!([PATH]).required(true)))
}

#[test]
fn verify_cmd() {
    cmd().debug_assert();
}
