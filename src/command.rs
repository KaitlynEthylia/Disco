use std::{fs, path::PathBuf};

use log::{error, info, warn};

use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

use clap::{value_parser, Parser};

use crate::Disco;

#[derive(Parser, Debug)]
//TODO disco2 vs disco vs disco-rpc
// Waiting on https://github.com/clap-rs/clap/issues/3221
#[command(author, version, about, long_about = None)]
struct Command {
	#[arg(
		short,
		long,
		env = "DISCO_CONFIG",
		value_name = "FILE",
		help = "Override the default configuration path.",
		default_value = get_path().into_os_string()
	)]
	config: PathBuf,

	#[arg(
		short = 'i',
		long,
		env = "DISCO_APPLICATION_ID",
		value_name = "ID",
		help = "Set the ID of the Discord application to connect to.",
		long_help = "Set the ID of the Discord application to connect to. \
		This value takes precedent over the value set in Lua.",
		value_parser = value_parser!(u64).range(10000000000000000..=999999999999999999)
	)]
	application_id: Option<u64>,

	#[arg(
		short,
		long,
		env = "DISCO_RETRY_AFTER",
		value_name = "DELAY",
		default_value_t = 0,
		help = "Retry after a failed connection.",
		long_help = "If connecting to Discord fails, keep retrying every DELAY \
		seconds."
	)]
	retry_after: usize,

	#[arg(
		short,
		long,
		env = "DISCO_QUIET",
		action = clap::ArgAction::Count,
        help = "Disables printing excess information.",
        long_help = "Disable printing information about the running process.
		Set twice to disable output completely, including errors.",
        value_parser = value_parser!(u8).range(0..=2)
	)]
	quiet: u8,

	#[arg(
		short,
		long,
		help = "Print the default configuration location.",
		long_help = "Halts normal execution and prints the location that Disco \
		will attempt to use for  configuration by default."
	)]
	print_config_path: bool,

	#[arg(short, long, help = "Parse the config but don't connect to Discord.")]
	dry_run: bool,

	#[cfg(feature = "unsafe")]
	#[arg(
		short,
		long,
		help = "Run the Lua VM in safe mode.",
		long_help = "Run the Lua VM in safe mode, meaning it will not be able \
		to load any C libraries. This option is only available when compiled \
		with the unsafe feature flag."
	)]
	safe: bool,
}

fn get_path() -> PathBuf {
	dirs::config_local_dir()
		.unwrap_or(PathBuf::new())
		.join("disco.lua")
}

fn validate_path(pathbuf: &PathBuf) -> bool {
	let path = pathbuf.as_path();
	path.exists() && path.is_file()
}

macro_rules! unwrap_error {
	($expr:expr, $msg:literal) => {
		match $expr {
			Ok(val) => val,
			Err(_) => {
				error!($msg);
				return None;
			},
		}
	};
}

pub fn init() -> Option<Disco> {
	let args = Command::parse();

	let log_level = match args.quiet {
		0 => LevelFilter::Debug,
		1 => LevelFilter::Warn,
		2.. => LevelFilter::Off,
	};

	TermLogger::init(
		log_level,
		Config::default(),
		TerminalMode::Mixed,
		ColorChoice::Auto,
	)
	.expect("Failed to setup logging");

	let valid_path = validate_path(&args.config);

	if args.print_config_path {
		info!("Config path is {}", args.config.display());
		if !valid_path {
			warn!("Config file does not exist");
		};
		return None;
	}

	if !valid_path {
		error!("Config file does not exist");
		return None;
	};

	let data = unwrap_error!(fs::read(&args.config), "Failed to open config file.");
	let data = unwrap_error!(String::from_utf8(data), "Config is not valid UTF-8.");

	return Some(Disco {
		retry_after: args.retry_after,
		dry_run: args.dry_run,
		application_id: args.application_id,
		config_data: data,
		#[cfg(feature = "unsafe")]
		safe: args.safe,
	});
}
