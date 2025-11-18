# Chop-Chop

## Project Setup

[Install rustup](https://rustup.rs/) and set up a standard Rust development environment.

Follow the instructions [here](https://gtk-rs.org/gtk4-rs/git/book/installation.html) and [here](https://gtk-rs.org/gtk4-rs/git/book/libadwaita.html) to install other system dependencies (a C compiler, GTK4 development libraries, and Libadwaita development libraries).

Install settings gschema by running the following from the top-level repo directory:

```bash
./install_schema.sh
```

Run this command after making any edits to `gschema.xml`.

ctrl+alt for inlay hints

## Building

The following command will build and run the application:

```bash
cargo run
```

## Style

Format code using `rustfmt` (default settings). Limit lines to 100 characters.

## Tests

To run unit tests:

```bash
cargo test
```

To run unit tests and show printed output:

```bash
cargo test -- --nocapture
```

## Todos

### Design Inspiration

- https://flathub.org/en/apps/io.github.nokse22.Exhibit
- https://flathub.org/en/apps/io.github.tfuxu.Halftone

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
- [x] Adjust column widths
- [x] Improve window scaling behavior
- [x] Carry changes over to parts pane (avoiding repeated code)
- [x] Show a graphic/instructions if there are no solver results
- [x] Add about page
- [x] Allow configuring the default units
- [x] Option to clear fields after adding
- [x] Disable delete button if nothing is selected
- [x] Fix mixed length format when terms are zero
- [x] Add support for 2D entries
- [x] Add price precision option
- [x] Add length precision option
- [x] Fix price formatting
- [x] Allow creating multiple windows (ctl+n)
- [x] Option to prompt before exiting with unsaved changes
- [x] Separate "save" and "save as" actions
- [x] Show filename in titlebar with an indicator if there are unsaved changes
- [x] Add option to open last project on restart
- [x] Add unit conversions
- [x] Fix result drawing
- [x] Allow changing draw/print font
- [x] Add print menu item and keyboard shortcut
- [x] Add basic keyboard shortcuts (for three button actions)
- [x] Setting for illustration scale (all the same, or fill space)
- [x] Add cut/blade width as a solver option
- [ ] Add support for 2D solvers
- [ ] Make a custom widget for length entries (unit + major + minor)

### Long-Term

- [ ] Show a suggestion drop-down for materials (existing materials)
- [x] Allow configuring fraction display format
- [x] Draw cut diagrams
- [ ] Make dialog animations consistent
- [ ] Option to merge rows if they share material+length+price (hash table)
- [ ] Show a popup if selecting a new row would discard changes (config option?)
- [ ] Define algorithm spec so people can add their own (external Python)
- [x] Allow sorting columns ascending/descending
- [ ] Allow filtering columns by vaule
- [ ] Support basic formulas for lengths
- [ ] Add welcome screen with project selection
- [x] Save/load projects to json
- [ ] Use a "jump" overlay (outline) in the report view
- [ ] Allow choice to minimize waste or cost
- [ ] Bulk deletions and updates
  - [ ] Toggle to enable bulk edit mode
  - [ ] Multi-select in the column view
  - [ ] Choose which field to update
- [ ] Find a way to make it more fun
- [ ] Deploy through aws or azure
