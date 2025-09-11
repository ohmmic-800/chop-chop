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

## Todos

### Short-Term

- [x] Split `window.rs` into multiple files (one per pane)
- [x] Finish editing UI
    - [x] Update entry widgets when selecting a new row
    - [x] The "update" button saves changes to the selected row
    - [x] The "add new" button saves changes to a new row
    - [x] Add "delete" button
    - [x] Discard pending changes when a new row is selected
- [x] Validate entry fields
    - [x] Highlight invalid entries with red or yellow border
    - [x] Don't allow saving changes if there is an invalid entry
- [x] Show a graphic/instructions if there are no rows
- [x] Parse fractions in length units
- [x] Use a split field for ft+in
- [x] Improve column value formatting (currency symbol for price, combined length column)
- [ ] Adjust column widths
- [ ] Show a suggestion drop-down for materials (existing materials)
- [ ] Add basic keyboard shortcuts (for three button actions)
- [ ] Carry changes over to parts pane (avoiding repeated code)
- [ ] Add unit conversions
- [ ] Improve window scaling behavior (no hard-coded widths)
- [ ] Show a graphic/instructions if there are no solver results

### Long-Term

- [ ] Allow configuring fraction display format
- [ ] Show a popup if selecting a new row would discard changes (config option?)
- [ ] Define algorithm spec so people can add their own
- [ ] Allow sorting columns ascending/descending
- [ ] Allow filtering columns by vaule
- [ ] Support basic formulas for lengths
- [ ] Add welcome screen with project selection
- [ ] Setting for illustration scale (all the same, or fill space)
- [ ] Save/load projects to json
- [ ] Allow configuring the default units
- [ ] Use a "jump" overlay (outline) in the report view
- [ ] Add cut/blade width as a solver option
- [ ] Allow choice to minimize waste or cost
