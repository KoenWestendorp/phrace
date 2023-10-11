use std::io::Read;

use terminal_size::{terminal_size, Height, Width};

#[derive(Debug, Clone)]
struct Axis {
    label: String,
}

impl Axis {
    fn new(label: String) -> Self {
        Self { label }
    }
}

#[derive(Debug, Default, Clone)]
struct Attributes {
    title: Option<String>,
    subtitle: Option<String>,
    xaxis: Option<Axis>,
    yaxis: Option<Axis>,
    _type: Option<String>,
    /// Attributes that are not relevant or unknown to this program.
    ///
    /// When an attribute is not recognized, the whole line is simply pushed into here.
    misc: Vec<String>,
}

/// Describes the data from an xvg file that is relevant for graphing to the terminal.
#[derive(Debug, Clone)]
pub struct Data {
    attributes: Attributes,
    cols: usize,
    rows: usize,
    array: Vec<f32>,
}

impl Data {
    // TODO: Consider returning an error when bad floats are encountered. This is very rare though,
    // and a panic may actually be the most appropriate course of action, even though it is ugly.
    /// Read xvg formatted data to create a new [`Data`].
    ///
    /// # Panics
    ///
    /// Panics if the data section of the xvg data contains content that cannot be parsed as
    /// [`f32`].
    fn from_xvg(xvg: &str) -> Self {
        let mut attributes = Attributes::default();
        let mut cols = None; // TODO: I don't love this but it'll do for now.
        let mut rows = 0;
        let mut array = Vec::new();
        for line in xvg.lines() {
            // Lines that start with a '#' are comments. Skip them.
            if line.starts_with('#') {
                continue;
            }

            // Lines that start with a '@' denote attributes. We store those.
            //
            // Warning: Excuse my horribly messy parsing code here...
            if line.starts_with('@') {
                // This unwrap is safe, since we just checked for it.
                let line = line.strip_prefix('@').unwrap().trim();
                if let Some((key, value)) = line.split_once('"') {
                    // We have a string field.
                    let mut key = key.trim().split_ascii_whitespace();
                    // If there is no closing double quote, something must really be awry with the
                    // data. We want to fail hard here.
                    let value = value
                        .strip_suffix('"')
                        .expect("expected trailing double quote in attribute");
                    match key.next() {
                        Some("xaxis") => {
                            if key.next() == Some("label") {
                                attributes.xaxis = Some(Axis::new(value.to_string()))
                            }
                        }
                        Some("yaxis") => {
                            if key.next() == Some("label") {
                                attributes.yaxis = Some(Axis::new(value.to_string()))
                            }
                        }
                        Some("title") => attributes.title = Some(value.to_string()),
                        Some("subtitle") => attributes.subtitle = Some(value.to_string()),
                        _other => attributes.misc.push(line.to_string()),
                    }
                }

                // Treat this as whitespace-separated items.
                let attr: Vec<_> = line.trim().split_ascii_whitespace().collect();
                let Some((&value, key)) = attr.split_last() else {
                    attributes.misc.push(line.to_string());
                    continue;
                };

                match key {
                    ["TYPE"] => attributes._type = Some(value.to_string()),
                    _other => attributes.misc.push(line.to_string()),
                }

                continue;
            }

            // Otherwise, we read the actual data.
            let values = line
                .trim()
                .split_ascii_whitespace()
                // If there are any items that cannot be parsed as floats, we ignore them. In case
                // of Ramachandran plots, the last column represents the specific residue that the
                // preceding x and y are associated with. We don't do anything with the residue
                // information, so we can discard it.
                // FIXME: What happens if there happens to be a row that for some other reason has
                // an non-float value and thus leads to a reading frame shift? For now, see the
                // debug_assert_eq below.
                .map(|v| v.parse::<f32>()).flatten();
            if cols.is_none() {
                cols = Some(values.clone().count())
            }
            debug_assert_eq!(values.clone().count(), cols.unwrap());
            array.extend(values);

            rows += 1;
        }

        Self {
            attributes,
            cols: cols.unwrap_or(0),
            rows,
            array,
        }
    }
}

impl Data {
    /// Return a column [`DataView`] from this [`Data`].
    pub fn col(&self, idx: usize) -> DataView<'_> {
        DataView::col(self, idx)
    }

    /// Return a row [`DataView`] from this [`Data`].
    pub fn row(&self, idx: usize) -> DataView<'_> {
        DataView::row(self, idx)
    }
}

#[derive(Debug, Clone, Copy)]
enum View {
    Col(usize),
    Row(usize),
}

/// A view into a column or a row of [`Data`].
#[derive(Debug, Clone, Copy)]
pub struct DataView<'d> {
    data: &'d Data,
    view: View,
    step: usize,
}

impl<'d> DataView<'d> {
    /// Create a new column [`DataView`] from [`Data`].
    pub(crate) fn col(data: &'d Data, idx: usize) -> Self {
        Self {
            data,
            view: View::Col(idx),
            step: 0,
        }
    }

    /// Create a new row [`DataView`] from [`Data`].
    pub(crate) fn row(data: &'d Data, idx: usize) -> Self {
        Self {
            data,
            view: View::Row(idx),
            step: 0,
        }
    }

    /// Returns the length of this [`DataView`].
    pub fn len(&self) -> usize {
        match self.view {
            View::Col(_) => self.data.rows,
            View::Row(_) => self.data.cols,
        }
    }

    /// Returns true if this [`DataView`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl DataView<'_> {
    /// Mean over the view.
    pub fn mean(&self) -> f32 {
        let n = self.len() as f32;
        self.sum::<f32>() / n
    }

    /// Varience over the view.
    pub fn variance(&self) -> f32 {
        let mean = self.mean();
        let n = self.len() as f32;
        self.map(|v| (v - mean).powi(2)).sum::<f32>() / n
    }

    /// Population standard deviation over the view.
    pub fn standard_deviation(&self) -> f32 {
        self.variance().sqrt()
    }

    /// Estimate of the standard error over the view.
    pub fn standard_error(&self) -> f32 {
        let n = self.len() as f32;
        self.standard_deviation() / n.sqrt()
    }

    /// Maximum value in the view.
    pub fn max_value(&self) -> f32 {
        // TODO: We assume that the view is not empty. That can lead to unexpected results.
        let mut max = f32::NEG_INFINITY;
        for v in *self {
            if v > max {
                max = v
            }
        }
        max
    }

    /// Minimum value in the view.
    pub fn min_value(&self) -> f32 {
        // TODO: We assume that the view is not empty. That can lead to unexpected results.
        let mut min = f32::INFINITY;
        for v in *self {
            if v < min {
                min = v
            }
        }
        min
    }
}

impl Iterator for DataView<'_> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.view {
            View::Col(col) => {
                if self.step >= self.data.rows {
                    return None;
                }
                let v = self
                    .data
                    .array
                    .get(self.data.cols * self.step + col)
                    .copied();
                self.step += 1;
                v
            }
            View::Row(row) => {
                if self.step >= self.data.cols {
                    return None;
                }
                let v = self
                    .data
                    .array
                    .get(self.data.cols * row + self.step)
                    .copied();
                self.step += 1;
                v
            }
        }
    }
}

fn graph(data: &Data, width: u16, height: u16) {
    assert!(width > 2);
    assert!(height > 3);
    let width = width as usize;
    let height = height as usize;
    let graph_width = width - 2; // Subtracting the width of the left-hand y-axis gutter.
    let graph_height = height - 3; // Subtracting the height for the title, subtitle, and x-axis.

    // TODO: We only graph the first data column, currently.
    let xs = data.col(0);
    let ys = data.col(1);

    let map = |min, max, a, size| (((a - min) * (size - 1) as f32) / (max - min)) as usize;
    let to_screen_x = |x| map(xs.max_value(), xs.min_value(), x, graph_width);
    let to_screen_y = |y| map(ys.max_value(), ys.min_value(), y, graph_height);

    let mut screen = vec![vec![0; graph_width]; graph_height];

    for (x, y) in xs.zip(ys) {
        let screen_x = to_screen_x(x);
        let screen_y = to_screen_y(y);
        screen[screen_y][screen_x] += 1;
    }

    let hi = *screen
        .concat()
        .iter()
        .max()
        .expect("screen size cannot be zero");
    let lo = *screen
        .concat()
        .iter()
        .filter(|&&v| v > 0)
        .min()
        .expect("screen size cannot be zero");
    let mut graph_rows = Vec::with_capacity(graph_height);
    for row in screen {
        let mut line = String::with_capacity(row.len());
        for &col in row.iter().rev() {
            if col > 0 {
                const PALETTE: &[u8; 9] = b".:-=+*#%@";
                let idx = (PALETTE.len() - 1) * (col - lo) / (hi - lo).max(1);
                line.push(PALETTE[idx] as char)
            } else {
                line.push(' ')
            }
        }
        graph_rows.push(line)
    }

    // Now onto the actual drawing.

    // Draw titles.
    if let Some(title) = &data.attributes.title {
        println!("{:^width$}", truncate(title, width));
    }
    if let Some(subtitle) = &data.attributes.subtitle {
        println!("{:^width$}", truncate(subtitle, width));
    }

    let ylabel = format!(
        "{:^graph_height$}",
        data.attributes
            .yaxis
            .as_ref()
            .map(|Axis { label }| truncate(label, graph_height))
            .unwrap_or("".to_string())
    );
    // The actual graph.
    for (row, ylabel_ch) in graph_rows.iter().zip(ylabel.chars()) {
        println!("{ylabel_ch} {row}")
    }

    // The x-axis label.
    if let Some(Axis { label }) = &data.attributes.xaxis {
        println!("{:^graph_width$}", truncate(label, graph_width));
    }
}

// TODO: A case for Cow?
/// If longer than `maxlen`, shorten a `&str` to be exactly the length `maxlen` and include into
/// that length a truncation symbol (`…`).
fn truncate(s: &str, maxlen: usize) -> String {
    const TRUNCATE_SYMBOL: char = '…';

    if s.len() > maxlen {
        // Truncate that thing!
        if maxlen > 0 {
            // There's room for adding the TRUNCATE_SYMBOL.
            let mut out = String::with_capacity(maxlen);
            s[..maxlen - 1].clone_into(&mut out);
            out.push(TRUNCATE_SYMBOL);
            out
        } else {
            "".to_string() // A 0-length string :/ Edge cases, you know...
        }
    } else {
        // No truncation necessary.
        s.to_string()
    }
}

fn usage(bin: &str) {
    const BIN: &str = env!("CARGO_BIN_NAME");
    const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    eprintln!("Please specify an xvg input path.");
    eprintln!("Usage: {bin} <input>");
    eprintln!();
    eprintln!("{BIN} {VERSION} by {AUTHORS}, 2023.");
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let bin = args.next().unwrap_or("dith".to_string());
    let Some(path) = args.next() else {
        usage(&bin);
        std::process::exit(1);
    };
    let mut file = std::fs::File::open(path)?;
    let mut xvg = String::new();
    file.read_to_string(&mut xvg)?;

    // TODO: From<BufReader> implementation?
    let data = Data::from_xvg(&xvg);

    // Present the graph :)
    let size = terminal_size();
    if let Some((Width(width), Height(height))) = size {
        if width < 5 || height < 7 {
            eprintln!("Terminal window is too small to present a meaningful graph.");
        } else {
            graph(&data, width, height - 2);
        }
    } else {
        eprintln!("Unable to get terminal size.");
    }

    // Nice little summary of the data.
    let ys = data.col(1);
    println!(
        "Summary:  {} items,  mean ± σ  {} ± {},  min … max  {} … {}",
        ys.len(),
        ys.mean(),
        ys.standard_deviation(),
        ys.min_value(),
        ys.max_value()
    );

    Ok(())
}
