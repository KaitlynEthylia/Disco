use std::{
	sync::mpsc::Sender,
	thread::{self, JoinHandle},
	time::Duration,
};

use mlua::prelude::*;

use crate::{
	activity::{ActivityData, ActivityDataTag, Button, Image, Timestamp},
	get_lua,
};

enum Variable<T> {
	Poll(u64),
	Listen,
	Static(T),
}

impl<'lua, T: FromLua<'lua>> FromLua<'lua> for Variable<T> {
	fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
		Ok(match lua_value.type_name() {
			"table" => {
				let tbl = LuaTable::from_lua(lua_value, lua)?;
				let rate = tbl.get(1);
				match rate {
					Ok(rate) => Self::Poll(rate),
					Err(_) => Self::Static(T::from_lua(LuaValue::Table(tbl), lua)?),
				}
			},
			"thread" => Self::Listen,
			_ => Self::Static(T::from_lua(lua_value, lua)?),
		})
	}
}

impl<T: Clone + for<'lua> FromLua<'lua> + Send + 'static> Variable<T> {
	fn watch<F: Fn(T) + Send + 'static>(
		self,
		name: &'static str,
		data: String,
		dofun: F,
	) -> Option<JoinHandle<()>> {
		match self {
			Self::Static(val) => {
				dofun(val);
				None
			},
			Self::Poll(rate) => Some(thread::spawn(move || {
				let lua = get_lua();
				lua.load(&data).exec().unwrap();
				let fun: LuaFunction = lua
					.globals()
					.get::<_, LuaTable>(name)
					.unwrap()
					.get(2)
					.unwrap();
				loop {
					let val: T = fun.call(()).unwrap();
					dofun(val);
					thread::sleep(Duration::from_secs(rate))
				}
			})),
			Self::Listen => Some(thread::spawn(move || {
				let lua = get_lua();
				lua.load(&data).exec().unwrap();
				let thread: LuaThread = lua.globals().get(name).unwrap();
				loop {
					match thread.resume::<_, Option<T>>(()) {
						Ok(Some(val)) => dofun(val),
						_ => break,
					}
				}
			})),
		}
	}
}

macro_rules! watchtype {
	($var:ident, $ty:ty, $name:ident, $send:ident, $ctx:ident, $env:ident) => {
		$env.get::<_, Variable<$ty>>($name)?
			.watch($name, $ctx.clone(), move |val| {
				$send.send(ActivityData::$var(val)).unwrap()
			})
	};
}

pub fn create_watcher(
	name: &'static str,
	send: Sender<ActivityData>,
	ctx: &String,
	env: &LuaTable,
	tag: &ActivityDataTag,
) -> Result<Option<JoinHandle<()>>, LuaError> {
	Ok(match tag {
		ActivityDataTag::Active => watchtype!(Active, bool, name, send, ctx, env),
		ActivityDataTag::State => watchtype!(State, String, name, send, ctx, env),
		ActivityDataTag::Details => watchtype!(Details, String, name, send, ctx, env),
		ActivityDataTag::Timestamp => watchtype!(Timestamp, Timestamp, name, send, ctx, env),
		ActivityDataTag::FirstButton => watchtype!(FirstButton, Button, name, send, ctx, env),
		ActivityDataTag::SecondButton => watchtype!(SecondButton, Button, name, send, ctx, env),
		ActivityDataTag::LargeImage => watchtype!(LargeImage, Image, name, send, ctx, env),
		ActivityDataTag::SmallImage => watchtype!(SmallImage, Image, name, send, ctx, env),
	})
}