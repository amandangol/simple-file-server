# Simple File Server

This is a simple file server implemented in Rust that serves files as a simple HTML document listing all directories and folders as links.

## Features

- Traverse up and into directories
- Read and view files
- Watch videos (with proper content-type headers)
- Properly decode special characters (e.g., CJK characters)
- Prevent backtracking beyond the server's root directory

## Requirements

- Rust (stable version)
- Cargo (Rust's package manager)

## Dependencies

This project uses minimal external dependencies to meet the challenge requirements:

- `std` (Rust standard library)
- `infer` (for file type detection)
- `url-escape` (for handling CJK and special characters in URLs)

No HTTP-related libraries (such as hyper or reqwest) are used in this implementation.

## Building and Running

1. Clone the repository:

   ```
   git clone https://github.com/your-username/simple-file-server.git
   cd simple-file-server
   ```

2. Build the project:

   ```
   cargo build --release
   ```

3. Run the server:

   ```
   cargo run --release [path]
   ```

   Replace `[path]` with the directory you want to serve. If no path is provided, the current directory will be used.

4. Open a web browser and navigate to `http://localhost:5500` to view the served directory.

## Usage Notes

- The server runs on `localhost` (127.0.0.1) on port 5500 by default.
- Directory listings are displayed as HTML pages with clickable links.
- Files can be viewed or downloaded depending on their type.
- Videos can be watched directly in the browser (if the browser supports the video format).
- Backtracking beyond the server's root directory is prohibited for security reasons.

## Security Considerations

- This server is intended for local use only and should not be exposed to the public internet without proper security measures.
- The server prevents access to directories outside of the specified root directory.
