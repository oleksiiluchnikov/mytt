# mytt

A simple and efficient time tracking tool for your daily workflow.

## Features

- [x] Log sessions with a custom shell script
- [ ] Task overview and management
- [ ] Reporting and analytics

## Usage

1. Install `mytt` using your preferred package manager.
2. Configure the `on_log.sh` script in `~/.config/mytt/` to suit your needs.
   <!-- 3. Run `mytt start` to begin a new session. -->
   <!-- 4. When you're done, run `mytt stop` to log the session. -->

### Example `on_log.sh`

```bash
#!/bin/bash

# Get the current date
DATE=$(date +'%Y-%m-%d')

# Get the session duration
DURATION="$1"

# Construct the log entry
LOG_ENTRY="- [$DURATION] $(date +'%H:%M') - Current Task"

# Append the log entry to your Obsidian daily note
echo "$LOG_ENTRY" >> "/path/to/obsidian/vault/$DATE.md"

# Open your Obsidian tasks file
nvim "/path/to/obsidian/vault/tasks.md" &&
open "obsidian://open?vault=Vault&file=tasks.md"
```

This example script appends the session duration and timestamp to your Obsidian daily note and opens your tasks file for easy task management.

## Contributing

Contributions are welcome! Please open an issue or log a pull request on the [GitHub repository](https://github.com/yourusername/mytt).
