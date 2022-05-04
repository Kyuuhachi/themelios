use std::{path::PathBuf, borrow::Cow};
use std::fs;
use std::io::Write as _;

use clap::StructOpt;
use kaiseki::ed6::Archive;

#[derive(Debug, Clone, clap::Parser)]
struct Cli {
	#[clap(flatten)]
	verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,

	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Command {
	/// Extract a single .dir/.dat archive.
	Extract {
		/// Overwrite the output directory if it already exists.
		#[clap(short, long)]
		force: bool,

		/// .dir file to read from. Corresponding .dat file must also exist.
		#[clap(value_hint=clap::ValueHint::FilePath)]
		dirfile: PathBuf,
		/// Directory to write extracted files to.
		#[clap(value_hint=clap::ValueHint::DirPath)]
		outdir: PathBuf,
	},

	/// Extract multiple .dir/.dat archives from a directory.
	///
	/// Each `<indir>/{file}.dir` will be extracted to `<outdir>/{file}`.
	ExtractAll {
		/// Overwrite the output directories if they already exist.
		#[clap(short, long)]
		force: bool,

		/// Directory containing .dir files to be extracted.
		#[clap(value_hint=clap::ValueHint::DirPath)]
		indir: PathBuf,
		/// Superdirectory to write extracted directories to.
		#[clap(value_hint=clap::ValueHint::DirPath)]
		outdir: PathBuf,
	},
}

fn main() -> eyre::Result<()> {
	let cli = Cli::parse();
	env_logger::Builder::new()
		.filter_level(cli.verbose.log_level_filter())
		.init();

	match cli.command {
		Command::Extract { force, dirfile, outdir } => {
			if !dirfile.is_file() || dirfile.extension().filter(|a| a == &"dir").is_none() {
				log::error!("Dirfile {} must be a .dir file", dirfile.display());
				return Ok(())
			}
			let datfile = dirfile.with_extension("dat");
			if !datfile.is_file() {
				log::error!("Datfile {} not found", datfile.display());
				return Ok(())
			}

			if outdir.exists() {
				if force {
					fs::remove_dir_all(&outdir)?;
				} else {
					log::error!("Output directory {} already exists (use -f to overwrite)", outdir.display());
					return Ok(())
				}
			}

			extract(&dirfile, &datfile, &outdir)?;
		},

		Command::ExtractAll { force, indir, outdir } => {
			if !indir.is_dir() {
				log::error!("Input directory {} not valid", indir.display());
				return Ok(())
			}

			for a in fs::read_dir(&indir)? {
				let a = a?;
				let dirfile = a.path();
				if dirfile.extension().filter(|a| a == &"dir").is_some() {
					let datfile = dirfile.with_extension("dat");
					if !datfile.is_file() {
						log::error!("{} not found", datfile.display());
						return Ok(())
					}

					let outdir = outdir.join(dirfile.file_stem().unwrap());

					if outdir.exists() {
						if force {
							fs::remove_dir_all(&outdir)?;
						} else {
							log::error!("Output directory {} already exists (use -f to overwrite)", outdir.display());
							return Ok(())
						}
					}

					extract(&dirfile, &datfile, &outdir)?;
				}
			}
		},
	}

	Ok(())
}

fn extract(dirfile: &PathBuf, datfile: &PathBuf, outdir: &PathBuf) -> eyre::Result<()> {
	fs::create_dir_all(&outdir)?;

	let arch = Archive::from_dir_dat(&dirfile, &datfile)?;
	let mut index = fs::File::create(outdir.join("index"))?;

	for e in arch.entries() {
		let outfile = outdir.join(e.display_name());
		let raw = arch.get(e.index).unwrap().1;
		log::info!("Extracting {} ({} → {})", outfile.display(), raw.len(), e.size);

		let (note, data) = if e.timestamp == 0 {
			(" e ", None)
		} else if raw.len() == e.size {
			("   ", Some(Cow::Borrowed(raw)))
		} else {
			match kaiseki::decompress::decompress(raw) {
				Ok(decompressed) => {
					("(C)", Some(Cow::Owned(decompressed)))
				}
				Err(e) => {
					log::warn!("  Decompression failed: {}", e);
					("(?)", Some(Cow::Borrowed(raw)))
				}
			}
		};

		if let Some(data) = &data {
			fs::write(&outfile, data)?;
			filetime::set_file_mtime(&outfile, filetime::FileTime::from_unix_time(e.timestamp as i64, 0))?;
		}

		writeln!(index, "{:4} {} {:?} ({} → {}; {})", e.index, note, e.name, raw.len(), data.map_or(0, |a| a.len()), e.size)?;
	}

	Ok(())
}
