use plotly::{
    common::{AxisSide, Font, HoverInfo, Marker, MarkerSymbol, Mode, Side, Title},
    layout::{Axis, Center, DragMode, Mapbox, MapboxStyle, Margin},
    Layout, Plot, Scatter, Scatter3D,
};

/// Supported plot items
pub enum PlotItem {
    /// 2D plot (1 or 2 y-axes), time domain or not
    Plot2d(Plot),
}

impl PlotItem {
    /// Generic 2D plot builder
    fn build_2d_plot<'a>(opts: Plot2dOpts) -> Plot {
        let layout = Layout::new()
            .title(Title::new(&opts.title).font(opts.title_font))
            .x_axis(
                Axis::new()
                    .title(Title::new(&opts.x_axis_title))
                    .zero_line(opts.zero_line.0)
                    .show_tick_labels(opts.show_xtick_labels)
                    .dtick(opts.dx_tick)
                    .tick_format(opts.x_tick_fmt),
            )
            .y_axis(
                Axis::new()
                    .title(Title::new(&opts.y_axis_title))
                    .zero_line(opts.zero_line.1),
            )
            .show_legend(opts.show_legend)
            .auto_size(opts.auto_size);
        let mut p = Plot::new();
        p.set_layout(layout);
        p
    }
    /// Standardized time domain 2D plot. We use MJD
    /// to represent long time series better.
    // TODO: it would be nice to maybe take advantage of the [Calendar] option
    // in case the time axis spans more than 1 month ?
    fn new_2d_time_axis(opts: Plot2dOpts) -> Self {
        Self::Plot2d(Self::build_2d_plot(opts))
    }
}

/// 2D Plot builder options
#[derive(Debug, Clone)]
pub struct Plot2dOpts {
    /// Plot title
    title: String,
    /// Plot title side
    title_side: Side,
    /// Plot title font
    title_font: Font,
    /// x axis title
    x_axis_title: String,
    /// y axis title
    y_axis_title: String,
    /// Plots a bold line @ (x=0, y=0)
    zero_line: (bool, bool),
    /// Whether this is visible in the Legend or not
    show_legend: bool,
    /// Auto size
    auto_size: bool,
    /// Display xtick labels or not
    show_xtick_labels: bool,
    /// dx
    dx_tick: f64,
    /// x tick format
    x_tick_fmt: String,
}

/// Default 2D plot options
impl Default for Plot2dOpts {
    fn default() -> Self {
        Self {
            title: Default::default(),
            title_side: Side::Top,
            title_font: Font::default(),
            x_axis_title: Default::default(),
            y_axis_title: Default::default(),
            zero_line: Default::default(),
            show_legend: Default::default(),
            auto_size: Default::default(),
            show_xtick_labels: Default::default(),
            dx_tick: Default::default(),
            x_tick_fmt: Default::default(), //TODO
        }
    }
}

impl Plot2dOpts {
    fn with_title(&self, title: &str) -> Self {
        let mut s = self.clone();
        s.title = title.to_string();
        s
    }
    fn with_x_title(&self, title: &str) -> Self {
        let mut s = self.clone();
        s.x_axis_title = title.to_string();
        s
    }
}
