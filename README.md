## Project Setup

[Install rustup](https://rustup.rs/) and set up a standard Rust development environment.

Follow the instructions [here](https://gtk-rs.org/gtk4-rs/git/book/installation.html) to install other system dependencies (a C compiler and GTK4 development libraries).

If using Visual Studio Code, install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension. See the VSCode [Rust language page](https://code.visualstudio.com/docs/languages/rust).

## Building

The following command will build and run the application:
```
cargo run
```

## Style

Format code using `rustfmt` (default settings). Limit lines to 100 characters.

## Tests

To run unit tests:
```
cargo test
```
To run unit tests and show printed output:
```
cargo test -- --nocapture
```
