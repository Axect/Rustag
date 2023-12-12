# Rustag

Rustag is a Rust-based command-line tool for organizing and tagging files on your system. It allows users to tag files with custom labels and retrieve them easily using a fuzzy search interface.

## Features

- **File Tagging**: Assign custom tags to your files for easy organization.
- **Fuzzy Search**: Easily find your files using a fuzzy search interface.
- **File Management**: Open files or their containing folders directly from the command line.

## Installation

To install Rustag, ensure you have Rust and Cargo installed on your system.
Then run the following command:

```bash
cargo install rustag
```

And for additional features, add the following to your `~/.bashrc` or `~/.zshrc`:

```bash
rtg() {
  RUSTAG=$(rustag $@)
  
  # RUSTAG contains "Error" then just print else cd $(RUSTAG)
  if [[ $RUSTAG == *"Error"* ]]; then
    echo $RUSTAG
  else
    cd $RUSTAG
  fi
}
```

Then run `rtg` to get started.

## Setup

Upon first run, Rustag will create a `.rustag` directory in your home folder to store its data. This includes a `tagfile` that maintains the tag information.

## Usage

### Tagging a File

To tag a file, run Rustag with the file name as an argument:

```bash
rtg filename.ext
```

Follow the prompts to select existing tags or create a new one.

### Retrieving Files

To view and open files associated with a tag:

1. Run Rustag without any arguments.
2. Select a tag from the list.
3. Choose a file to open or a related action.

## Data Structure

Rustag uses several custom data structures:

- `AGFile`: Represents a tagged file, including metadata such as creation date and file path.
- `AGTag`: Represents a tag, associated with multiple `AGFile` instances.
- `AGTagList`: Manages the collection of tags and their associated files.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.
