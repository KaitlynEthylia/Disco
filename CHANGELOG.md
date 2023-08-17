# 0.2.0

## Added

 - Lua values may now be set to functions that will return the desired
   type. This prevents code being run multiple times when multiple Lua
   VMs are spawned.

 - The `watch` function has been added as a builtin helper function
   for executing a shell command and following the output. The
   function takes a shell command as the first arg, and optionally a
   value to use if the command fails to start.

 - When built with the `unsafe` feature flag, the `--safe` flag can be
   passed to the command line to only load safe libraries anyway.

 - The `--dry-run` command line flag can be passed to prevent
   attempting to connect to Discord.

 - Fish and Zsh completions are now provided with the release.

 - Systemd, Runit, and OpenRC template services can be found in the
   [etc](/etc) directory.

## Changed

 - ClientID has been renamed to ApplicationID to better reflect
 Discord's own usage and reduce ambiguity.

 - Output has been cleaned up, making use of
 [simplelog](https://lib.rs/simplelog).

 - Command help text has been improved.

## Fixed

 - The output binary is now named `disco`. Previously it was
   mistakenly called `disco-rpc` by default.
