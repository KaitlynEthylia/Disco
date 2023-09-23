use std::sync::mpsc;
use std::time::Duration;
use std::{process, thread};

use discord_rich_presence::activity::{Activity, Assets, Button, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use log::{error, info, warn};
use mlua::Lua;

use crate::activity::ActivityData;
use crate::activity::ActivityDataTag::*;
use crate::activity::{Button as DiscoButton, Image, Timestamp};
use crate::lua::create_watcher;

mod activity;
mod command;
mod library;
mod lua;

#[derive(Debug)]
pub struct Disco {
	pub retry_after: usize,
	pub dry_run: bool,
	pub application_id: Option<u64>,
	pub config_data: String,
	#[cfg(feature = "unsafe")]
	pub safe: bool,
}

pub fn get_lua(#[cfg(feature = "unsafe")] safe: bool) -> Lua {
	#[cfg(not(feature = "unsafe"))]
	let lua = Lua::new();
	#[cfg(feature = "unsafe")]
	let lua = if !safe {
		unsafe { Lua::unsafe_new() }
	} else {
		Lua::new()
	};
	if let Err(_) = lua.load(library::DISCO_LIB).exec() {
		warn!("Failed to load provided builtin functions, only the Lua standard library will be available");
	};
	lua
}

fn main() {
	let args = match command::init() {
		Some(disco) => disco,
		None => process::exit(0),
	};

	let lua = get_lua(
		#[cfg(feature = "unsafe")]
		args.safe,
	);
	lua.load(&args.config_data).exec().unwrap();
	let env = lua.globals();

	let application_id = match args.application_id {
		Some(id) => id.to_string(),
		None => match env.get("ApplicationID") {
			Ok(application_id) => application_id,
			Err(_) => {
				error!("No application id available. Set ApplicationID in disco.lua or with --application-id");
				process::exit(0);
			},
		},
	};

	info!("Application ID: {application_id}");

	let mut client = match args.dry_run {
		false => Some(
			DiscordIpcClient::new(&application_id).expect("Failed to create Discord IPC Client"),
		),
		true => {
			info!("Performing dry run. Won't attempt to connect to Discord");
			None
		},
	};

	match client {
		Some(_) => println!("AAA"),
		None => println!("BBB"),
	};

	if let Some(ref mut client) = client {
		let mut ret = client.connect();
		match args.retry_after {
			0 => {
				if let Err(_) = ret {
					error!("Failed to connect to Discord IPC. Please make sure Discord is open.");
					process::exit(0);
				}
			},
			n => {
				while let Err(_) = ret {
					warn!("Failed to connect to Discord IPC. Retrying in {n} seconds...");
					thread::sleep(Duration::from_secs(n as u64));
					ret = client.connect();
				}
			},
		};
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
		let ret = create_watcher(
			name,
			send.clone(),
			&args.config_data,
			&env,
			&ty,
			#[cfg(feature = "unsafe")]
			args.safe,
		);
		if let Err(_) = ret {
			warn!("No value for {name}");
		}
	});

	let mut activity = DiscoActivity::new();

	loop {
		match recv.recv() {
			Ok(val) => {
				info!("New Value: {val:?}");
				match val {
					ActivityData::Active(val) => {
						if let Some(ref mut client) = client {
							if !val {
								let _ = client.clear_activity();
							}
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
				if let Some(ref mut client) = client {
					activity.process(client);
				}
			},
			Err(_) => {
				info!("Exiting...");
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

	fn process(&self, client: &mut DiscordIpcClient) {
		println!("processing: {self:#?}");
		if self.active {
			let mut activity = Activity::new();

			if let Some(val) = &self.state {
				activity = activity.state(val);
				print!("0");
			}
			if let Some(val) = &self.details {
				activity = activity.details(val);
				print!("1");
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
				print!("2");
			}

			let mut buttons = Vec::with_capacity(2);
			let mut buttons_set = false;
			if let Some(val) = &self.button1 {
				buttons.push(Button::new(&val.text, &val.url));
				buttons_set = true;
				print!("3");
			}
			if let Some(val) = &self.button2 {
				buttons.push(Button::new(&val.text, &val.url));
				buttons_set = true;
				print!("4");
			}
			if buttons_set {
				activity = activity.buttons(buttons);
				print!("5");
			}

			let mut assets = Assets::new();
			let mut assets_set = false;
			if let Some(val) = &self.large_image {
				if let Some(val) = &val.text {
					assets = assets.large_text(&val);
				}
				assets = assets.large_image(&val.asset);
				assets_set = true;
				print!("6");
			}
			if let Some(val) = &self.small_image {
				if let Some(val) = &val.text {
					assets = assets.small_text(&val);
				}
				assets = assets.small_image(&val.asset);
				assets_set = true;
				print!("7");
			}
			if assets_set {
				activity = activity.assets(assets);
				print!("8");
			}

			println!("setting activity");
			client.set_activity(activity).unwrap();
			print!("9");
		}
	}
}
