use adw::prelude::*;
use gtk::cairo;
use gtk::glib::clone;
use pango::{FontDescription, units_from_double, units_to_double};
use pangocairo::functions::{create_layout, show_layout};

use crate::modeling::{CutList, Part, SubSolution, Supply};
use crate::size::FractionFormat;

pub trait DisplayBlock {
    /// Displays contents within a widget
    fn display(&self, b: &gtk::Box);

    /// Returns the height of the drawn content
    fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64;

    /// Returns the height of the content without doing anyting to the cairo Context
    fn height(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64;
}

#[derive(Default)]
pub struct DisplayEngine {
    /// A list of blocks to draw
    blocks: Vec<Box<dyn DisplayBlock>>,

    /// Track the start and end indices of sections
    sections: Vec<(usize, usize)>,

    /// Index of the last unclosed section
    open_section: Option<usize>,

    /// Block indices where page breaks are needed
    pagination: Option<Vec<usize>>,
}

impl DisplayEngine {
    const MARGIN_HORIZONTAL: f64 = 20.0;
    const MARGIN_VERTICAL: f64 = 10.0;
    const MARGIN_EXPANDER_HORIZONTAL: i32 = 24;
    const MARGIN_EXPANDER_VERTICAL: i32 = 16;

    pub fn append_block(&mut self, block: Box<dyn DisplayBlock>) {
        self.blocks.push(block);
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.sections.clear();
        self.open_section = None;
        self.pagination = None;
    }

    pub fn display(&self, b: &gtk::Box) {
        while let Some(c) = b.last_child() {
            b.remove(&c);
        }

        // Index into section list
        let mut k = 0;

        let mut active_box = b.clone();
        for (i, block) in self.blocks.iter().enumerate() {
            if let Some((_, j)) = self.sections.get(k)
                && *j == i
            {
                // End section
                active_box = b.clone();
                k += 1;
            }
            if let Some((j, _)) = self.sections.get(k)
                && *j == i
            {
                // Start section (use block as section header)
                let label_widget = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .build();
                block.display(&label_widget);
                active_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .margin_start(Self::MARGIN_EXPANDER_HORIZONTAL)
                    .margin_end(Self::MARGIN_EXPANDER_HORIZONTAL)
                    .margin_top(Self::MARGIN_EXPANDER_VERTICAL)
                    .margin_bottom(Self::MARGIN_EXPANDER_VERTICAL)
                    .build();
                let expander = gtk::Expander::builder()
                    .label_widget(&label_widget)
                    .child(&active_box)
                    .build();
                b.append(&expander);
            } else {
                // Only show the block if it was not used as a section header
                block.display(&active_box);
            }
        }
    }

    /// Returns the height of the drawn content
    pub fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        self.draw_blocks(self.blocks.as_slice(), c, f, w)
    }

    pub fn draw_page(&self, c: &cairo::Context, f: &FontDescription, w: f64, i: usize) {
        let pagination = self
            .pagination
            .as_ref()
            .expect("Must call paginate() before draw_page()");
        let j = if i != 0 { pagination[i - 1] } else { 0 };
        let k = if i < self.n_pages() - 1 {
            pagination[i]
        } else {
            self.blocks.len()
        };
        self.draw_blocks(&self.blocks[j..k], c, f, w);
    }

    pub fn end_section(&mut self) {
        let i = self.open_section.expect("No active section");
        self.sections.push((i, self.blocks.len()));
        self.open_section = None;
    }

    pub fn n_pages(&self) -> usize {
        let pagination = self
            .pagination
            .as_ref()
            .expect("Must call paginate() before n_pages()");
        pagination.len() + 1
    }

    pub fn paginate(&mut self, c: &cairo::Context, f: &FontDescription, w: f64, h: f64) {
        let mut pagination = Vec::new();
        let mut y = 0.0;
        for (i, block) in self.blocks.iter().enumerate() {
            let b = block.height(c, f, w);
            if (y != 0.0) && (y + b + 2.0 * Self::MARGIN_VERTICAL > h) {
                pagination.push(i);
                y = 0.0;
            } else {
                y += b;
            }
        }
        self.pagination = Some(pagination);
    }

    pub fn start_section(&mut self) {
        self.open_section = Some(self.blocks.len());
    }

    // Convenience methods:

    pub fn append_cut_diagram(
        &mut self,
        cut_list: &CutList,
        sub_solution: &SubSolution,
        max_length: Option<f64>,
    ) {
        self.append_block(Box::new(CutDiagram::from(
            cut_list,
            sub_solution,
            max_length,
        )))
    }

    pub fn append_header_1(&mut self, text: &str) {
        self.append_block(Box::new(Header1::from(text)));
    }

    pub fn append_header_2(&mut self, text: &str) {
        self.append_block(Box::new(Header2::from(&text)))
    }

    pub fn append_header_3(&mut self, text: &str) {
        self.append_block(Box::new(Header3::from(&text)))
    }

    pub fn append_paragraph(&mut self, text: &str) {
        self.append_block(Box::new(Paragraph::from(&text)))
    }

    pub fn append_table(&mut self, rows: Vec<Vec<String>>, alignment: Vec<gtk::Align>) {
        self.append_block(Box::new(Table::from(rows, alignment)));
    }

    /// Returns the height of the drawn content
    fn draw_blocks(
        &self,
        blocks: &[Box<dyn DisplayBlock>],
        c: &cairo::Context,
        f: &FontDescription,
        w: f64,
    ) -> f64 {
        let mut y = Self::MARGIN_VERTICAL;
        for block in blocks {
            c.move_to(Self::MARGIN_HORIZONTAL, y);
            y += block.draw(c, f, w - Self::MARGIN_HORIZONTAL * 2.0);
        }
        y + Self::MARGIN_VERTICAL
    }
}

#[derive(Clone)]
pub struct CutDiagram {
    supply: Supply,
    parts: Vec<Part>,
    max_length: Option<f64>,
}

impl CutDiagram {
    const MARGIN_TOP: f64 = 12.0;
    const MARGIN_LABEL: f64 = 8.0;
    const MARGIN_LENGTH: f64 = 8.0;
    const MARGIN_BOTTOM: f64 = 24.0;
    const TICK_SIZE: f64 = 8.0;

    pub fn from(cut_list: &CutList, sub_solution: &SubSolution, max_length: Option<f64>) -> Self {
        let supply = sub_solution.supplies[cut_list.supply_index].clone();
        let parts = cut_list
            .part_indices
            .iter()
            .map(|i| sub_solution.parts[*i].clone())
            .collect();
        CutDiagram {
            supply,
            parts,
            max_length,
        }
    }
}

impl DisplayBlock for CutDiagram {
    fn display(&self, b: &gtk::Box) {
        let drawing_area = gtk::DrawingArea::builder().build();
        drawing_area.set_draw_func(clone!(
            #[strong(rename_to = block)]
            self,
            move |d, c, w, _| {
                let style = adw::StyleManager::default();
                let font = FontDescription::from_string(&format!(
                    "{} {}",
                    style.document_font_name(),
                    "12"
                ));
                c.set_source_color(&d.color());
                let h = block.draw(c, &font, w as f64);
                d.set_height_request(h as i32);
            },
        ));
        b.append(&drawing_area);
        drawing_area.queue_draw();
    }

    fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        if !c.has_current_point().unwrap() {
            c.move_to(0.0, 0.0);
        }
        let supply_length: f64 = self.supply.length.to_meters_f64();
        let w = match self.max_length {
            Some(length) => w * supply_length / length,
            None => w,
        };

        c.rel_move_to(0.0, Self::MARGIN_TOP);

        // Draw labels
        let (x, y) = c.current_point().unwrap();
        let mut h_1 = 0.0;
        for part in self.parts.iter() {
            let dx = part.length.to_meters_f64() / supply_length * w;
            let h = draw_text(c, f, dx, &part.name, true);
            h_1 = if h > h_1 { h } else { h_1 };
            c.rel_move_to(dx, 0.0);
        }
        c.move_to(x, y + h_1);

        c.rel_move_to(0.0, Self::MARGIN_LABEL);

        // Draw ticks
        let (x, y) = c.current_point().unwrap();
        c.rel_line_to(0.0, Self::TICK_SIZE);
        for part in self.parts.iter() {
            let dx = part.length.to_meters_f64() / supply_length * w;
            c.rel_move_to(dx, -Self::TICK_SIZE);
            c.rel_line_to(0.0, Self::TICK_SIZE);
        }
        c.move_to(x + w, y);
        c.rel_line_to(0.0, Self::TICK_SIZE);
        c.stroke().unwrap();
        c.move_to(x, y + Self::TICK_SIZE);

        // Draw line
        let (x, y) = c.current_point().unwrap();
        c.rel_move_to(0.0, -Self::TICK_SIZE / 2.0);
        c.rel_line_to(w, 0.0);
        c.stroke().unwrap();
        c.move_to(x, y);

        c.rel_move_to(0.0, Self::MARGIN_LENGTH);

        // Draw lengths
        let (x, y) = c.current_point().unwrap();
        let mut h_2 = 0.0;
        for part in self.parts.iter() {
            // TODO: Use correct fraction format here
            let l = part.length.format(&FractionFormat::Mixed);
            let dx = part.length.to_meters_f64() / supply_length * w;
            let h = draw_text(c, f, dx, &l, true);
            h_2 = if h > h_2 { h } else { h_2 };
            c.rel_move_to(dx, 0.0);
        }
        c.move_to(x, y + h_2);

        Self::MARGIN_TOP
            + h_1
            + Self::MARGIN_LABEL
            + Self::TICK_SIZE
            + Self::MARGIN_LENGTH
            + h_2
            + Self::MARGIN_BOTTOM
    }

    fn height(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        let mut h_1 = 0.0;
        for part in self.parts.iter() {
            let h = draw_text(c, f, w, &part.name, false);
            h_1 = if h > h_1 { h } else { h_1 };
        }
        let mut h_2 = 0.0;
        for part in self.parts.iter() {
            // TODO: Use correct fraction format here
            let l = part.length.format(&FractionFormat::Mixed);
            let h = draw_text(c, f, w, &l, true);
            h_2 = if h > h_2 { h } else { h_2 };
        }
        Self::MARGIN_TOP
            + h_1
            + Self::MARGIN_LABEL
            + Self::TICK_SIZE
            + Self::MARGIN_LENGTH
            + h_2
            + Self::MARGIN_BOTTOM
    }
}

pub struct Header1 {
    text: String,
}

impl Header1 {
    const MARGIN_TOP: f64 = 24.0;
    const MARGIN_UNDERLINE: f64 = 2.0;
    const MARGIN_BOTTOM: f64 = 0.0;
    const TEXT_SIZE: &str = "x-large";

    pub fn from(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }

    fn markup(&self) -> String {
        format!(
            "<span size='{}'><b>{}</b></span>",
            Self::TEXT_SIZE,
            self.text
        )
    }
}

impl DisplayBlock for Header1 {
    fn display(&self, b: &gtk::Box) {
        let label = gtk::Label::builder()
            .label(&self.text)
            .halign(gtk::Align::Start)
            .build();
        label.add_css_class("display-header-1");
        b.append(&label);
    }

    fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        c.rel_move_to(0.0, Self::MARGIN_TOP);
        let h = draw_text(c, f, w, &self.markup(), true);
        c.rel_move_to(0.0, h + Self::MARGIN_UNDERLINE);
        c.rel_line_to(w, 0.0);
        c.stroke().unwrap();
        h + Self::MARGIN_TOP + Self::MARGIN_UNDERLINE + Self::MARGIN_BOTTOM
    }

    fn height(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        draw_text(c, f, w, &self.markup(), false)
            + Self::MARGIN_TOP
            + Self::MARGIN_UNDERLINE
            + Self::MARGIN_BOTTOM
    }
}

pub struct Header2 {
    text: String,
}

impl Header2 {
    const MARGIN_TOP: f64 = 12.0;
    const MARGIN_BOTTOM: f64 = 0.0;
    const TEXT_SIZE: &str = "large";

    pub fn from(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }

    fn markup(&self) -> String {
        format!(
            "<span size='{}'><b>{}</b></span>",
            Self::TEXT_SIZE,
            self.text
        )
    }
}

impl DisplayBlock for Header2 {
    fn display(&self, b: &gtk::Box) {
        let label = gtk::Label::builder()
            .label(&self.text)
            .halign(gtk::Align::Start)
            .build();
        label.add_css_class("display-header-2");
        b.append(&label);
    }

    fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        c.rel_move_to(0.0, Self::MARGIN_TOP);
        let h = draw_text(c, f, w, &self.markup(), true);
        h + Self::MARGIN_TOP + Self::MARGIN_BOTTOM
    }

    fn height(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        draw_text(c, f, w, &self.markup(), false) + Self::MARGIN_TOP + Self::MARGIN_BOTTOM
    }
}

pub struct Header3 {
    text: String,
}

impl Header3 {
    const MARGIN_TOP: f64 = 12.0;
    const MARGIN_BOTTOM: f64 = 0.0;
    const TEXT_SIZE: &str = "medium";

    pub fn from(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }

    fn markup(&self) -> String {
        format!(
            "<span size='{}'><b>{}</b></span>",
            Self::TEXT_SIZE,
            self.text
        )
    }
}

impl DisplayBlock for Header3 {
    fn display(&self, b: &gtk::Box) {
        let label = gtk::Label::builder()
            .label(&self.text)
            .halign(gtk::Align::Start)
            .build();
        label.add_css_class("display-header-3");
        b.append(&label);
    }

    fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        c.rel_move_to(0.0, Self::MARGIN_TOP);
        let h = draw_text(c, f, w, &self.markup(), true);
        h + Self::MARGIN_TOP + Self::MARGIN_BOTTOM
    }

    fn height(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        draw_text(c, f, w, &self.markup(), false) + Self::MARGIN_TOP + Self::MARGIN_BOTTOM
    }
}

pub struct Paragraph {
    text: String,
}

impl Paragraph {
    const MARGIN_TOP: f64 = 12.0;
    const MARGIN_BOTTOM: f64 = 0.0;

    pub fn from(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }
}

impl DisplayBlock for Paragraph {
    fn display(&self, b: &gtk::Box) {
        let label = gtk::Label::builder()
            .label(&self.text)
            .halign(gtk::Align::Start)
            .build();
        label.add_css_class("display-paragraph");
        b.append(&label);
    }

    fn draw(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        c.rel_move_to(0.0, Self::MARGIN_TOP);
        let h = draw_text(c, f, w, &self.text, true);
        h + Self::MARGIN_TOP + Self::MARGIN_BOTTOM
    }

    fn height(&self, c: &cairo::Context, f: &FontDescription, w: f64) -> f64 {
        draw_text(c, f, w, &self.text, false) + Self::MARGIN_TOP + Self::MARGIN_BOTTOM
    }
}

fn draw_text(c: &cairo::Context, f: &FontDescription, w: f64, s: &str, show: bool) -> f64 {
    let p = create_layout(c);
    p.set_font_description(Some(f));
    p.set_width(units_from_double(w));
    p.set_wrap(pango::WrapMode::Word);
    p.set_markup(&s);
    if show {
        show_layout(c, &p);
    }

    // https://docs.gtk.org/Pango/method.Layout.get_extents.html
    units_to_double(p.extents().1.height())
}

pub struct Table {
    rows: Vec<Vec<String>>,
    alignment: Vec<gtk::Align>,
}

impl Table {
    pub fn from(rows: Vec<Vec<String>>, alignment: Vec<gtk::Align>) -> Self {
        Self { rows, alignment }
    }
}

impl DisplayBlock for Table {
    fn display(&self, b: &gtk::Box) {
        let grid = gtk::Grid::builder()
            .column_spacing(32)
            .row_spacing(8)
            .build();
        for (i, row) in self.rows.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                let align = self
                    .alignment
                    .get(j)
                    .expect(&format!("Alignment not provided for column {}", j));
                let label = gtk::Label::builder().halign(*align).build();
                label.set_markup(value);
                grid.attach(&label, j as i32, i as i32, 1, 1);
            }
        }
        b.append(&grid);
    }

    fn draw(&self, _c: &cairo::Context, _f: &FontDescription, _w: f64) -> f64 {
        // TODO
        0.0
    }

    fn height(&self, _c: &cairo::Context, _f: &FontDescription, _w: f64) -> f64 {
        // TODO
        0.0
    }
}
