complete -c disco -f
complete -c disco -s 'V' -l 'version' -d "Print version"
complete -c disco -s 'h' -l 'help' -d "Print help"
complete -c disco -s 'c' -l 'config' -d "Override the default configuration path." -F
complete -c disco -s 'i' -l 'application-id' -d "Set the ID of the Discord application to connect to."
complete -c disco -s 'r' -l 'retry-after' -d "Retry after a failed connection."
complete -c disco -s 'q' -l 'quiet' -d "Disables printing excess information." -x -a "0 1 2"
complete -c disco -s 'p' -l 'print-config-path' -d "Print the default configuration location."
complete -c disco -s 'd' -l 'dry-run' -d "Parse the config but don't connect to Discord."
