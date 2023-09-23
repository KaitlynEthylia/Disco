# 0.2.1

I originally planned this as a 0.3.0 release with some more fixes, but
as this currently doesn't work on windows at all, I decided it would
be better to release this now.

## Changed

 - `watch` function now accepts a function as a third argument to
   manipulate each line

## Fixed

 - Switched to vendored Lua application.

 - Systemd Unit missing `[Install]` section.

 - Fixed setting values to functions
