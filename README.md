<div align="center">

[Installation](#installation) |
[Usage](#usage) |
[Configuration](#configuration) |
[Example](#example)

# Disco

[![Github](https://img.shields.io/badge/Github-KaitlynEthylia%2FDisco-cec2fc?logo=github&style=for-the-badge)](https://github.com/KaitlynEthylia/Disco)
[![Crates.io](https://img.shields.io/crates/v/disco-rpc?color=%23f7b679&logo=rust&style=for-the-badge)](https://crates.io/crates/disco-rpc)
[![Unlicense](https://img.shields.io/crates/l/terny?color=bfdfff&logo=unlicense&style=for-the-badge)](https://unlicense.org/)

Disco is a customisable client for Discord rich presence using
simple Lua configuration.

</div>

<a id="installation" />

## Installation

Disco can be installed from crates.io or manually installed from
GitHub.

### Creates.io

Assuming you already have the rust toolchain installed, you can simply
install disco by running

```sh
cargo install disco-rpc
```

To allow Lua to call external C libraries, the `unsafe` feature flag
also needs to be enabled.

```sh
cargo install disco-rpc --features unsafe
```

### GitHub

Prebuilt binaries are currently provided for x86_64-linux-gnu both
with and without the unsafe flag. Disco can also be built from source
via cargo

```sh
git clone https://github.com/KaitlynEthylia/Disco
cd Disco
cargo install --path .
```

As with the crates.io installation, `--features unsafe` my optionally
be appended to the final command.

<a id="usage" />

## Usage

Running the command `disco` will automatically look for a Lua file to
read configuration from as described in the
[configuration section](#configuration). There are a handful of
arguments that may be passed to the command to adjust how it runs.

Naturally, the `-h, --help` and `-V, --version` flags exist. As well,
there is a `-p --print-config-path` flag, which will prevent the
programme from running and print the location that it would otherwise
look for a configuration file.
The other flags that actually affect the programme are:

**-c, --config <FILE>**: Override the default configuration path.
<br />
**-i, --application-id <ID>**: Set the ID of the Discord application to connect to.
<br />
**-r, --retry-after <DELAY>**: Retry after a failed connection.
<br />
**-q, --quiet ...**: Disables printing excess information.
<br />
**-d, --dry-run**: Parse the config but don't connect to Discord.
<br />

<a id="configuration" />

## Configuration

Disco will look for a config file by first seeing if one has been
given in the command line via the `-c` flag. If not, it will look at
the `DISCO_CONFIG` environment variable. If that is not set, it will
then check `$XDG_CONFIG_HOME/disco.lua` and lastly `$HOME/disco.lua`.

If it still cannot find a config file, it will error. There is no
default configuration.

The rich presence is made up of a series of variable that can be set
in the config file.

| Variable | Type | Description |
| -------- | ---- | ----------- |
| Active | boolean | Whether or not to display the rich presence. |
| Details | string | The first line of the rich presence. |
| State | string | The second line of the rich presence. |
| Timestaamp | Timestamp | A timestamp to count up or count down, appears in the same location as State. Note that the timestamp can be buggy. If the rich presence is not displaying, you may have set this value incorrectly, remove it and see if that solves the issue. |
| LargeImage | Image | The main image on the left hand side of the presence. |
| SmallImage | Image | The image in the bottom left corner of the large image. |
| Button1 | Button | A button that can direct to a URL. |
| Button2 | Button | A second button that can direct to a URL. Discord only supports a maximum of 2. |

the types Timestamp, Image, and Button are not part of lua, they each
represent:
| Type | Explanation | Examples |
| ---- | ----------- | -------- |
| Timestamp | a table containing an option `start` key, and an optional `_end` key. alternatively, a single integer equivelelent to setting `start` but not `_end`. | `{ start = 12345, _end = 23456 }` or `10000`. |
| Image | a URL to the image or a table whose first element is a URL, and with a key `text` representing the text that appears when hovering over the image. | `'https://example.com/image.png'` or `{ 'https://example.com/image.png', text = "Some Image" }`. |
| Button | a URL to be directed to, or a table whose first element is the text to display on the image, and with a key `url` which is the url to be directed to. | `'https://example.org'` or `{ 'Example Website', url = 'https://example.org' }`. |

Each of these variables can be set in 3 different ways:

- It can be set simply by assigning the variable the value. We will
call this a 'static' variable, because its value will not change
throughout the programme.

```lua
State = "Some Text"
```

- It can be set via a table. The second element being a function that
returns the value, and the first element being the number of seconds
between successive calls of this function. This we will call a
'polled' variable.

```lua
State = { 60, function()
    return os.date("%H:%M")
end }
```

Here it runs the `os.date()` function every 60 seconds to get the
system time.

- Lastly, it can be set via a coroutine. This coroutine will
continually be resumed, and should yield a value to set each time.

```lua
State = coroutine.create(function()
    for i = 1, 10, 1 do
        coroutine.yield("Number: " .. i)
    end
end)
```

In this example, the coroutine counts up to ten and then stops
returning. Once it stops, the final value remains.
This example is contrived, but a more complex example can be seen
in [the example section](#example)

Additionally. As well as assigning a value directly, a function can be
given which will immediately return the value. This ensured that the
value is evaluated only once, despite the application having to launch
multiple Lua VMs.

Disco may also call external lua libraries, however, if that
library requires C libraries, the `unsafe` feature flag will need to
be enabled.

```lua
-- requires the `http` library to be installed, as well as the
-- `unsafe` feature to be enabled.
local request = require 'http.request'

local _, stream = request.new_from_uri('https://api.rot26.org/encrypt/test'):go()
State = stream:get_body_as_string()
```

<a id="example"/>

## Example

Here is my personal configuration that I use on my Arch Linux +
Hyprland setup so show some details. This isn't exactly what I use
but I have modified it slightly to demonstrate some features that
I didn't use, such as polling.

```lua
-- ID of the application I created for ArchLinux via
-- https://discord.com/developers/applications
ApplicationID = 1137762526541656105

-- Display the rich presence
Active = true

-- Show what time it is on my machine on the first line, rerun
-- the function every 60 seconds.
Details = { 60, function()
	return "My Time: " .. os.date("%H:%M")
end }

-- Plug into the Hyprland socket to show which window is currently
-- active, and update every time I switch window.
State = coroutine.create(function()
	local handle = io.popen([[
		socat -u UNIX-CONNECT:/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock - |
		stdbuf -o0 awk -F '>>|,' '/^activewindow>>/{ $2 = toupper(substr($2, 1, 1)) substr($2, 2); print $2}'
	]])
	if not handle then return end
	for line in handle:lines() do
		coroutine.yield("Using: " .. line)
	end
end)

-- Display the Arch Linux logo
LargeImage = {
	'https://seeklogo.com/images/A/arch-linux-logo-3C25E68BA9-seeklogo.com.png',
	text = 'Arch Linux',
}

-- Include the Hyprland logo in the corner
SmallImage = {
	'https://i.imgur.com/PanwaBQ.png',
	text = 'Hyprland',
}

-- A really cool button
Button1 = {
	'Press This Button!',
	url = 'https://youtu.be/dQw4w9WgXcQ',
}
```

### The Result

![Discord Rich Presence](/etc/assets/example.png)
