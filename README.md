## Project Setup

[Install rustup](https://rustup.rs/) and set up a standard Rust development environment.

Follow the instructions [here](https://gtk-rs.org/gtk4-rs/git/book/installation.html) and [here](https://gtk-rs.org/gtk4-rs/git/book/libadwaita.html) to install other system dependencies (a C compiler, GTK4 development libraries, and Libadwaita development libraries).

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

## Implementation Notes/Todos

- Setting to show all illustrations at the same scale
- Save/load project to json file
- Allow the unit system to change between items
- Specify the default unit system in settings
- Print report or save as PDF
- Overlay "jump to material" list in report view
- Solver options:
    - Algorithm
    - Blade width
    - Choice to minimize waste or cost
