#[derive(Debug, Clone)]
pub enum ActivityDataTag {
	Active,
	State,
	Details,
	Timestamp,
	FirstButton,
	SecondButton,
	LargeImage,
	SmallImage,
}

#[derive(Debug, Clone)]
pub enum ActivityData {
	Active(bool),
	State(String),
	Details(String),
	Timestamp(Timestamp),
	FirstButton(Button),
	SecondButton(Button),
	LargeImage(Image),
	SmallImage(Image),
}

use mlua::{Error::FromLuaConversionError, FromLua, Lua, Result, Table, Value};

#[derive(Debug, Clone)]
pub struct Image {
	pub asset: String,
	pub text: Option<String>,
}

impl<'lua> FromLua<'lua> for Image {
	fn from_lua(lua_value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
		let typename = lua_value.type_name();
		match typename {
			"table" => {
				let table: Table = Table::from_lua(lua_value, lua)?;
				Ok(Image {
					asset: table.get(1)?,
					text: table.get("text")?,
				})
			},
			"string" => Ok(Image {
				asset: String::from_lua(lua_value, lua)?,
				text: None,
			}),
			_ => Err(FromLuaConversionError {
				from: typename,
				to: "Image",
				message: None,
			}),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Button {
	pub text: String,
	pub url: String,
}

impl<'lua> FromLua<'lua> for Button {
	fn from_lua(lua_value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
		let typename = lua_value.type_name();
		match typename {
			"table" => {
				let table: Table = Table::from_lua(lua_value, lua)?;
				Ok(Button {
					text: table.get(1)?,
					url: table.get("url")?,
				})
			},
			"string" => {
				let url = String::from_lua(lua_value, lua)?;
				Ok(Button {
					text: url.clone(),
					url,
				})
			},
			_ => Err(FromLuaConversionError {
				from: typename,
				to: "Button",
				message: None,
			}),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Timestamp {
	pub start: Option<i64>,
	pub end: Option<i64>,
}

impl<'lua> FromLua<'lua> for Timestamp {
	fn from_lua(lua_value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
		let typename = lua_value.type_name();
		match typename {
			"table" => {
				let table: Table = Table::from_lua(lua_value, lua)?;
				Ok(Timestamp {
					start: table.get("start")?,
					end: table.get("_end")?,
				})
			},
			"number" => Ok(Timestamp {
				start: Option::from_lua(lua_value, lua)?,
				end: None,
			}),
			_ => Err(FromLuaConversionError {
				from: typename,
				to: "Timestamp",
				message: None,
			}),
		}
	}
}
