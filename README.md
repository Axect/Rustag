# Rustag

Rustag is a Rust-based command-line tool for bookmarking directories on your system. It allows users to assign custom aliases to directories and navigate to them easily using a fuzzy search interface.

## Features

- **Directory Bookmarking**: Assign custom aliases to your directories for easy organization.
- **Fuzzy Search**: Easily find your bookmarks using a fuzzy search interface with alias and path displayed together.
- **Directory Navigation**: Open directories in terminal or file manager directly from the command line.

## Installation

To install Rustag, ensure you have Rust and Cargo installed on your system.
Then run the following command:

```bash
cargo install rustag
```

And for additional features, add the following to your shell configuration file:

**For Bash/Zsh** (`~/.bashrc` or `~/.zshrc`):

```bash
rtg() {
  local output=$(rustag "$@")
  local last_line=$(echo "$output" | tail -n 1)

  # Check if the last line is a valid directory path
  if [[ -d "$last_line" ]]; then
    cd "$last_line"
  else
    echo "$output"
  fi
}
```

**For Fish** (`~/.config/fish/config.fish`):

```fish
function rtg
  set -l output (rustag $argv)
  set -l last_line (echo $output | tail -n 1)

  # Check if the last line is a valid directory path
  if test -d "$last_line"
    cd "$last_line"
  else
    echo "$output"
  end
end
```

Then run `rtg` to get started.

## Setup

Upon first run, Rustag will create a `.rustag` directory in your home folder to store its data. This includes a `bookmarks` file that maintains the bookmark information.

## Usage

### Adding a Bookmark

To bookmark the current directory, run Rustag without arguments and select "Add bookmark":

```bash
rtg
```

Follow the prompts to:
1. Select "Add bookmark" from the menu
2. Enter a custom alias for the current directory

### Viewing and Managing Bookmarks

To view and manage your bookmarks:

1. Run `rtg` without any arguments
2. Select "View bookmarks" from the menu
3. Browse your bookmarks with fuzzy search (each entry shows alias and full path)
4. Select a bookmark to perform actions:
   - **Open in Terminal**: Navigate to the directory (cd)
   - **Open in File Manager**: Open the directory in your system's file manager
   - **Edit alias**: Change the alias of the bookmark
   - **Remove bookmark**: Delete the bookmark

### Example Workflow

```bash
# Navigate to a project directory
cd ~/projects/myproject

# Add a bookmark
rtg
# Select: Add bookmark
# Enter alias: myproj

# Later, from anywhere
rtg
# Select: View bookmarks
# You'll see fuzzy searchable list:
#   myproj -> /home/user/projects/myproject
#   docs -> /home/user/documents
#   ...
# Select: myproj -> /home/user/projects/myproject
# Select: Open in Terminal
# Now you're in ~/projects/myproject
```

## Data Structure

Rustag uses two custom data structures:

- **`Bookmark`**: Represents a bookmarked directory, including:
  - `alias`: User-defined alias for the directory
  - `folder_path`: Absolute path to the directory
  - `created_at`: Timestamp when the bookmark was created
  - `last_accessed`: Timestamp when the bookmark was last accessed (optional)

- **`BookmarkList`**: Manages the collection of bookmarks using:
  - `bookmarks`: HashMap mapping aliases to Bookmark instances for O(1) lookup
  - `aliases`: Sorted vector of aliases for display purposes

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
