#[forbid(unsafe_code)]

mod field;
mod judges;
mod ui;

use clap::Parser;
use directories::ProjectDirs;

fn parse_density(s: &str) -> Result<f32, &'static str> {
    let f = s.parse().map_err(|_| "invalid number")?;
    if f > 1.0 || f < 0.0 {
        return Err("density out of range");
    }
    Ok(f)
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, short, default_value = "0.22", help = "The density of mines, between 0 and 1.", parse(try_from_str = parse_density))]
    density: f32,
    #[clap(long, short, help = "Ensure the board is always solvable without 'guessing'.")]
    solvable: bool,
    #[clap(long, short, default_value = "global", arg_enum)]
    judge: judges::Judge,
    #[clap(long, short, help = "See what the solver sees.")]
    cheat: bool,
    #[clap(long, short, help = "Save automatically after every click.")]
    autosave: bool,
    #[clap(long, help = "Delete and recreate the save file.")]
    reset: bool,
    #[clap(
        help = "The path to the save file. Will be created if it doesn't exist. Defaults to the value of MINEFAIR_SAVE if set, or to a reasonable platform-dependent config folder.",
        env = "MINEFAIR_SAVE",
        parse(from_os_str)
    )]
    save_path: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let path = args.save_path.clone().unwrap_or(
        match ProjectDirs::from("", "", "minefair") {
            Some(p) => p.data_dir().join("save.minefair"),
            None => {
                eprintln!("couldn't find a save file path to use. please pass a path as an argument or set MINEFAIR_SAVE");
                std::process::exit(1);
            },
        }
    );
    if path.is_dir() {
        eprintln!("is a directory");
        std::process::exit(1);
    }
    std::fs::create_dir_all(path.parent().unwrap()).expect("failed creating directories");

    ui::game_loop(args, path).unwrap();
}
