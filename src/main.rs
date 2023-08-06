use std::sync::mpsc;
use std::time::Duration;
use std::{env, path::PathBuf};
use std::{fs, thread};

use clap::Parser;

use dirs::config_dir;
use discord_rich_presence::activity::{Activity, Assets, Button, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use mlua::Lua;

use crate::activity::ActivityData;
use crate::activity::ActivityDataTag::*;
use crate::activity::{Button as DiscoButton, Image, Timestamp};
use crate::lua::create_watcher;

mod activity;
mod lua;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Command {
	/// Overrides the default path to look for configuration.
	#[arg(short, long)]
	config: Option<PathBuf>,

	/// Sets the ID of the application to connect as. Takes prescedent over Lua configuration.
	#[arg(short = 'i', long)]
	client_id: Option<String>,

	/// If connecting to Discord fails, retry after DELAY seconds.
	#[arg(short, long, value_name = "DELAY", default_value_t = 0)]
	retry_after: u64,

	/// Don't print any text to the console.
	#[arg(short, long)]
	quiet: bool,

	#[arg(short, long)]
	print_config_path: bool,
}

pub fn get_lua() -> Lua {
	#[cfg(not(feature = "unsafe"))]
	return Lua::new();
	#[cfg(feature = "unsafe")]
	unsafe {
		return Lua::unsafe_new();
	};
}

fn main() {
	let args = Command::parse();
	let path = match args.config {
		Some(path) => path,
		None => env::var("DISCO_CONFIG")
			.map(|val| PathBuf::from(val))
			.unwrap_or(
				config_dir()
					.expect(
						"Could not find a place to look for config. Please
                    specify a file in the command line or set
                    DISCO_CONFIG variable.",
					)
					.join("disco.lua"),
			),
	};

	if args.print_config_path {
		println!("{:?}", path);
		return;
	}

	let data = fs::read(&path).expect(&format!("Failed to read config file at {path:?}"));
	let file = String::from_utf8(data).expect("Contents of config file is not valid UTF-8");

	let lua = get_lua();
	lua.load(&file).exec().unwrap();
	let env = lua.globals();

	let client_id = match args.client_id {
		Some(id) => id,
		None => env
			.get("ClientID")
			.expect("No client id available. Set ClientID in {path} or with --client-id"),
	};

	if !args.quiet {
		println!("Client ID: {client_id}");
	}

	let mut client =
		DiscordIpcClient::new(&client_id).expect("Failed to create Discord IPC Client");
	let mut ret = client.connect();
	match args.retry_after {
		0 => {
			if let Err(_) = ret {
				panic!("Failed to connect to Discord IPC. Please make sure Discord is open.");
			}
		},
		n => {
			while let Err(_) = ret {
				println!("Failed to connect to Discord IPC. Retrying in {n} seconds...");
				thread::sleep(Duration::from_secs(n));
				ret = client.connect();
			}
		},
	};

	let (send, recv) = mpsc::channel::<ActivityData>();

	let values = vec![
		("State", State),
		("Details", Details),
		("Timestamp", Timestamp),
		("Button1", FirstButton),
		("Button2", SecondButton),
		("LargeImage", LargeImage),
		("SmallImage", SmallImage),
		("Active", Active),
	];

	values.iter().for_each(|(name, ty)| {
		let ret = create_watcher(name, send.clone(), &file, &env, &ty);
		if let Err(_) = ret {
			if !args.quiet {
				println!("No value for {name}");
			}
		}
	});

	let mut activity = DiscoActivity::new();

	loop {
		match recv.recv() {
			Ok(val) => {
				if !args.quiet {
					println!("New Value: {val:?}");
				}
				match val {
					ActivityData::Active(val) => {
						if !val {
							let _ = client.clear_activity();
						}
						activity.active = val
					},
					ActivityData::State(val) => activity.state = Some(val),
					ActivityData::Details(val) => activity.details = Some(val),
					ActivityData::Timestamp(val) => activity.timestamp = Some(val),
					ActivityData::FirstButton(val) => activity.button1 = Some(val),
					ActivityData::SecondButton(val) => activity.button2 = Some(val),
					ActivityData::LargeImage(val) => activity.large_image = Some(val),
					ActivityData::SmallImage(val) => activity.small_image = Some(val),
				}
				activity.process(&mut client, args.quiet);
			},
			Err(_) => {
				if !args.quiet {
					println!("Exiting...");
				}
				break;
			},
		}
	}
}

#[derive(Debug, Clone)]
struct DiscoActivity {
	active: bool,
	state: Option<String>,
	details: Option<String>,
	timestamp: Option<Timestamp>,
	button1: Option<DiscoButton>,
	button2: Option<DiscoButton>,
	large_image: Option<Image>,
	small_image: Option<Image>,
}

impl DiscoActivity {
	fn new() -> Self {
		Self {
			active: false,
			state: None,
			details: None,
			timestamp: None,
			button1: None,
			button2: None,
			large_image: None,
			small_image: None,
		}
	}

	fn process(&self, client: &mut DiscordIpcClient, quiet: bool) {
		if self.active {
			let mut activity = Activity::new();

			if let Some(val) = &self.state {
				activity = activity.state(val);
			}
			if let Some(val) = &self.details {
				activity = activity.details(val);
			}

			if let Some(val) = &self.timestamp {
				let mut timestamps = Timestamps::new();
				if let Some(val) = val.start {
					timestamps = timestamps.start(val);
				}
				if let Some(val) = val.end {
					timestamps = timestamps.end(val);
				}
				activity = activity.timestamps(timestamps);
			}

			let mut buttons = Vec::with_capacity(2);
			let mut buttons_set = false;
			if let Some(val) = &self.button1 {
				buttons.push(Button::new(&val.text, &val.url));
				buttons_set = true;
			}
			if let Some(val) = &self.button2 {
				buttons.push(Button::new(&val.text, &val.url));
				buttons_set = true;
			}
			if buttons_set {
				activity = activity.buttons(buttons);
			}

			let mut assets = Assets::new();
			let mut assets_set = false;
			if let Some(val) = &self.large_image {
				if let Some(val) = &val.text {
					assets = assets.large_text(&val);
				}
				assets = assets.large_image(&val.asset);
				assets_set = true;
			}
			if let Some(val) = &self.small_image {
				if let Some(val) = &val.text {
					assets = assets.small_text(&val);
				}
				assets = assets.small_image(&val.asset);
				assets_set = true;
			}
			if assets_set {
				activity = activity.assets(assets);
			}

			if !quiet {
				println!("DEBUG: {self:#?}");
			}
			client.set_activity(activity).unwrap();
		}
	}
}
