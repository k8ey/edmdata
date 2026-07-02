use bson::DateTime;
use chrono::{Datelike, Months, Timelike};
use derive_more::From;
use itertools::Itertools;
use svg::{
    Document, Node,
    node::element::{Group, Path, Rectangle, Text, path::Data},
};

use crate::{data::edm::EarlyDayMotion, query::Signature};

pub const WIDTH_HEIGHT_RATIO: (usize, usize) = (3, 1);

pub const WIDTH_HEIGHT_BASE: usize = 1024;

pub const DEFAULT_LINES: usize = 13;

pub const LIGHT_GREY: &str = "#707070";

pub const DULL_GREY: &str = "#505050";

pub const DARK_GREY: &str = "#303030";

pub const DULL_PURPLE: &str = "#805080";

pub const FONT_FAMILY: &str = "sans-serif";

pub const FONT_SIZE_TINY: usize = 7;

pub const FONT_SIZE_SMALL: usize = 12;

pub const FONT_SIZE_MEDIUM: usize = 22;

pub const FONT_SIZE_LARGE: usize = 30;

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackedGraphLabels {
    pub labels: Vec<(String, String)>,
}

impl StackedGraphLabels {
    pub fn new(labels: Vec<(String, String)>) -> Self {
        Self { labels }
    }
}

impl From<&StackedGraph> for StackedGraphLabels {
    fn from(value: &StackedGraph) -> Self {
        let labels = value
            .graphs
            .iter()
            .enumerate()
            .map(|(i, graph)| {
                (
                    graph.colour.clone(),
                    "#".to_owned() + i.to_string().as_str(),
                )
            })
            .collect();

        Self::new(labels)
    }
}

impl From<&mut StackedGraph> for StackedGraphLabels {
    fn from(value: &mut StackedGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<StackedGraph> for StackedGraphLabels {
    fn from(value: StackedGraph) -> Self {
        let value = value.fill_gaps();

        Self::from(&value)
    }
}

impl From<&StackedGraphLabels> for Group {
    fn from(value: &StackedGraphLabels) -> Self {
        let mut group = Group::new();

        for (i, (colour, label)) in value.labels.iter().enumerate() {
            group = group.add(
                Rectangle::new()
                    .set("width", "2%")
                    .set("height", "60%")
                    .set(
                        "x",
                        format!(
                            "{}%",
                            100.0 * (i + 1) as f64 / (value.labels.len() as f64 + 1.0) - 2.5
                        ),
                    )
                    .set("y", "20%")
                    .set("fill", colour.as_str()),
            );

            group = group.add(
                Text::new(label)
                    .set(
                        "x",
                        format!(
                            "{}%",
                            100.0 * (i + 1) as f64 / (value.labels.len() as f64 + 1.0) + 3.0 - 2.5
                        ),
                    )
                    .set("y", "50%")
                    .set("fill", colour.as_str())
                    .set("font-family", "sans-serif")
                    .set("font-size", format!("{}px", FONT_SIZE_LARGE))
                    .set("font-weight", "bold")
                    .set("dominant-baseline", "middle"),
            );
        }

        group
    }
}

impl From<StackedGraphLabels> for Group {
    fn from(value: StackedGraphLabels) -> Self {
        Self::from(&value)
    }
}

impl From<&mut StackedGraphLabels> for Group {
    fn from(value: &mut StackedGraphLabels) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphYAxisNumbers {
    pub points: Vec<(usize, usize)>,
    pub width: usize,
    pub height: usize,
    pub lines: usize,
    pub colour: String,
    pub font_size: usize,
}

impl GraphYAxisNumbers {
    pub fn new(
        points: Vec<(usize, usize)>,
        width: usize,
        height: usize,
        lines: usize,
        colour: impl Into<String>,
        font_size: usize,
    ) -> Self {
        Self {
            points,
            width,
            height,
            lines,
            colour: colour.into(),
            font_size,
        }
    }

    pub fn text(&self, y: usize) -> Text {
        Text::new(y.to_string())
            .set("x", "50%")
            .set("y", self.height - y)
            .set("fill", self.colour.as_str())
            .set("font-family", FONT_FAMILY)
            .set("font-size", format!("{}px", self.font_size))
            .set("text-anchor", "end")
            .set("dominant-baseline", "middle")
    }
}

impl From<&StackedGraph> for GraphYAxisNumbers {
    fn from(value: &StackedGraph) -> Self {
        Self::new(
            value
                .stack_graph(
                    value.graphs.len(),
                    &value.graphs.iter().rev().next().unwrap(),
                )
                .collect(),
            value.max_x(),
            value.max_y(),
            DEFAULT_LINES,
            LIGHT_GREY,
            FONT_SIZE_TINY,
        )
    }
}

impl From<&mut StackedGraph> for GraphYAxisNumbers {
    fn from(value: &mut StackedGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<StackedGraph> for GraphYAxisNumbers {
    fn from(value: StackedGraph) -> Self {
        let value = value.fill_gaps();

        Self::from(&value)
    }
}

impl From<&LineGraph> for GraphYAxisNumbers {
    fn from(value: &LineGraph) -> Self {
        Self::new(
            value.points.clone(),
            value.max_x(),
            value.max_y(),
            DEFAULT_LINES,
            LIGHT_GREY,
            FONT_SIZE_TINY,
        )
    }
}

impl From<&mut LineGraph> for GraphYAxisNumbers {
    fn from(value: &mut LineGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<LineGraph> for GraphYAxisNumbers {
    fn from(value: LineGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&BarGraph> for GraphYAxisNumbers {
    fn from(value: &BarGraph) -> Self {
        Self::new(
            value.points.clone(),
            value.max_x(),
            value.max_y(),
            DEFAULT_LINES,
            LIGHT_GREY,
            FONT_SIZE_TINY,
        )
    }
}

impl From<&mut BarGraph> for GraphYAxisNumbers {
    fn from(value: &mut BarGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<BarGraph> for GraphYAxisNumbers {
    fn from(value: BarGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&GraphYAxisNumbers> for Document {
    fn from(value: &GraphYAxisNumbers) -> Self {
        Document::new().add(Group::from(value)).set(
            "viewBox",
            format!("-8 -8 {} {}", value.height + 16, value.height + 16),
        )
    }
}

impl From<GraphYAxisNumbers> for Document {
    fn from(value: GraphYAxisNumbers) -> Self {
        Self::from(&value)
    }
}

impl From<&mut GraphYAxisNumbers> for Document {
    fn from(value: &mut GraphYAxisNumbers) -> Self {
        Self::from(&*value)
    }
}

impl From<&GraphYAxisNumbers> for Group {
    fn from(value: &GraphYAxisNumbers) -> Self {
        let mut group = Group::new();

        // An attempt to prevent overlap.
        for i in 1..=(value.height - value.height / value.lines) {
            if i % (value.height / value.lines) != 0 {
                continue;
            }

            group = group.add(value.text(i));
        }

        group.add(value.text(value.height))
    }
}

impl From<GraphYAxisNumbers> for Group {
    fn from(value: GraphYAxisNumbers) -> Self {
        Self::from(&value)
    }
}

impl From<&mut GraphYAxisNumbers> for Group {
    fn from(value: &mut GraphYAxisNumbers) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphYAxisLines {
    pub points: Vec<(usize, usize)>,
    pub width: usize,
    pub height: usize,
    pub lines: usize,
    pub colour: String,
}

impl GraphYAxisLines {
    pub fn new(
        points: Vec<(usize, usize)>,
        width: usize,
        height: usize,
        lines: usize,
        colour: impl Into<String>,
    ) -> Self {
        Self {
            points,
            width,
            height,
            lines,
            colour: colour.into(),
        }
    }

    pub fn path(&self, y: usize) -> Path {
        let data = Data::new()
            .move_to((0, self.height - y))
            .line_to((self.width, self.height - y));

        Path::new()
            .set("stroke", self.colour.as_str())
            .set("d", data)
    }
}

impl From<&StackedGraph> for GraphYAxisLines {
    fn from(value: &StackedGraph) -> Self {
        Self::new(
            value
                .stack_graph(
                    value.graphs.len(),
                    &value.graphs.iter().rev().next().unwrap(),
                )
                .collect(),
            value.max_x(),
            value.max_y(),
            DEFAULT_LINES,
            DULL_GREY.to_owned() + "70",
        )
    }
}

impl From<&mut StackedGraph> for GraphYAxisLines {
    fn from(value: &mut StackedGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<StackedGraph> for GraphYAxisLines {
    fn from(value: StackedGraph) -> Self {
        let value = value.fill_gaps();

        Self::from(&value)
    }
}

impl From<&LineGraph> for GraphYAxisLines {
    fn from(value: &LineGraph) -> Self {
        Self::new(
            value.points.clone(),
            value.max_x(),
            value.max_y(),
            DEFAULT_LINES,
            DULL_GREY.to_owned() + "70",
        )
    }
}

impl From<&mut LineGraph> for GraphYAxisLines {
    fn from(value: &mut LineGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<LineGraph> for GraphYAxisLines {
    fn from(value: LineGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&BarGraph> for GraphYAxisLines {
    fn from(value: &BarGraph) -> Self {
        Self::new(
            value.points.clone(),
            value.max_x(),
            value.max_y(),
            DEFAULT_LINES,
            DULL_GREY.to_owned() + "70",
        )
    }
}

impl From<&mut BarGraph> for GraphYAxisLines {
    fn from(value: &mut BarGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<BarGraph> for GraphYAxisLines {
    fn from(value: BarGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&GraphYAxisLines> for Document {
    fn from(value: &GraphYAxisLines) -> Self {
        Document::new().add(Group::from(value)).set(
            "viewBox",
            format!("-8 -8 {} {}", value.width + 16, value.height + 16),
        )
    }
}

impl From<GraphYAxisLines> for Document {
    fn from(value: GraphYAxisLines) -> Self {
        Self::from(&value)
    }
}

impl From<&mut GraphYAxisLines> for Document {
    fn from(value: &mut GraphYAxisLines) -> Self {
        Self::from(&*value)
    }
}

impl From<&GraphYAxisLines> for Group {
    fn from(value: &GraphYAxisLines) -> Self {
        let mut group = Group::new();

        // An attempt to prevent overlap.
        for i in 1..=(value.height - value.height / value.lines) {
            if i % (value.height / value.lines) != 0 {
                continue;
            }

            group = group.add(value.path(i));
        }

        group.add(value.path(value.height))
    }
}

impl From<GraphYAxisLines> for Group {
    fn from(value: GraphYAxisLines) -> Self {
        Self::from(&value)
    }
}

impl From<&mut GraphYAxisLines> for Group {
    fn from(value: &mut GraphYAxisLines) -> Self {
        Self::from(&*value)
    }
}

// TODO: Merge structs with similar elements into one.
#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphXAxisDashes {
    pub points: Vec<(usize, usize)>,
    pub width: usize,
    pub height: usize,
    pub interval: (usize, usize),
    pub colour: String,
}

impl GraphXAxisDashes {
    pub fn new(
        points: Vec<(usize, usize)>,
        width: usize,
        height: usize,
        interval: (usize, usize),
        colour: impl Into<String>,
    ) -> Self {
        Self {
            points,
            width,
            height,
            interval,
            colour: colour.into(),
        }
    }

    pub fn rect(&self, x: usize) -> Rectangle {
        Rectangle::new()
            .set("x", x)
            .set("width", "0.1%")
            .set("height", "100%")
            .set("fill", self.colour.as_str())
    }
}

impl From<&StackedGraph> for GraphXAxisDashes {
    fn from(value: &StackedGraph) -> Self {
        Self::new(
            value
                .stack_graph(
                    value.graphs.len(),
                    &value.graphs.iter().rev().next().unwrap(),
                )
                .collect(),
            value.max_x(),
            value.max_y(),
            (0, value.max_x() / DEFAULT_LINES),
            DULL_GREY.to_owned() + "70",
        )
    }
}

impl From<&mut StackedGraph> for GraphXAxisDashes {
    fn from(value: &mut StackedGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<StackedGraph> for GraphXAxisDashes {
    fn from(value: StackedGraph) -> Self {
        let value = value.fill_gaps();

        Self::from(&value)
    }
}

impl From<&BarGraph> for GraphXAxisDashes {
    fn from(value: &BarGraph) -> Self {
        Self::new(
            value.points.clone(),
            value.max_x(),
            value.max_y(),
            (0, value.max_x() / DEFAULT_LINES),
            DULL_GREY.to_owned() + "70",
        )
    }
}

impl From<&mut BarGraph> for GraphXAxisDashes {
    fn from(value: &mut BarGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<BarGraph> for GraphXAxisDashes {
    fn from(value: BarGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&LineGraph> for GraphXAxisDashes {
    fn from(value: &LineGraph) -> Self {
        Self::new(
            value.points.clone(),
            value.max_x(),
            value.max_y(),
            (0, value.max_x() / DEFAULT_LINES),
            DULL_GREY.to_owned() + "70",
        )
    }
}

impl From<&mut LineGraph> for GraphXAxisDashes {
    fn from(value: &mut LineGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<LineGraph> for GraphXAxisDashes {
    fn from(value: LineGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&GraphXAxisDashes> for Document {
    fn from(value: &GraphXAxisDashes) -> Self {
        Document::new().add(Group::from(value)).set(
            "viewBox",
            format!("-8 -8 {} {}", value.width + 16, value.height + 16),
        )
    }
}

impl From<GraphXAxisDashes> for Document {
    fn from(value: GraphXAxisDashes) -> Self {
        Self::from(&value)
    }
}

impl From<&mut GraphXAxisDashes> for Document {
    fn from(value: &mut GraphXAxisDashes) -> Self {
        Self::from(&*value)
    }
}

impl From<&GraphXAxisDashes> for Group {
    fn from(value: &GraphXAxisDashes) -> Self {
        let mut group = Group::new();

        for x in 1..=value.width {
            if x < value.interval.0 {
                continue;
            }

            if (x - value.interval.0) % value.interval.1 != 0 {
                continue;
            }

            group = group.add(value.rect(x));
        }

        group
    }
}

impl From<GraphXAxisDashes> for Group {
    fn from(value: GraphXAxisDashes) -> Self {
        Self::from(&value)
    }
}

impl From<&mut GraphXAxisDashes> for Group {
    fn from(value: &mut GraphXAxisDashes) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupportingSignaturesOverTimeGraph {
    pub graph: LineGraph,
    pub colours: Vec<String>,
    pub start_dt: DateTime,
    pub start_date: String,
    pub end_date: String,
    pub title: String,
}

impl SupportingSignaturesOverTimeGraph {
    pub const TIME_DIVISOR: usize = 1000 * 60 * 60 * 24;

    pub const DEFAULT_COLOURS: &[&str] = &[
        "#800000", "#804000", "#808000", "#408000", "#008040", "#008080", "#004080", "#400080",
        "#800080", "#800040",
    ];

    pub fn new(
        graph: LineGraph,
        colours: Vec<String>,
        start_dt: DateTime,
        start_date: impl Into<String>,
        end_date: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            graph,
            colours,
            start_dt,
            start_date: start_date.into(),
            end_date: end_date.into(),
            title: title.into(),
        }
    }
}

impl From<&Vec<(EarlyDayMotion, Vec<Signature>)>> for SupportingSignaturesOverTimeGraph {
    fn from(value: &Vec<(EarlyDayMotion, Vec<Signature>)>) -> Self {
        let min_x = value
            .iter()
            .map(|(edm, ..)| edm.date_tabled)
            .sorted()
            .next()
            .map(DateTime::timestamp_millis)
            .unwrap_or(0) as usize;

        let points = value
            .iter()
            .sorted_by(|a, b| a.0.date_tabled.cmp(&b.0.date_tabled))
            .map(|(edm, signatures)| {
                (
                    (edm.date_tabled.timestamp_millis() as usize - min_x) / Self::TIME_DIVISOR,
                    signatures.len(),
                )
            })
            .collect();

        let start_date = value
            .iter()
            .map(|(edm, ..)| edm.date_tabled)
            .sorted()
            .next();

        let end_date = value
            .iter()
            .map(|(edm, ..)| edm.date_tabled)
            .sorted()
            .rev()
            .next()
            .map(|dt| dt.to_chrono().format("%d/%m/%Y").to_string())
            .unwrap_or(String::new());

        Self::new(
            LineGraph::new(points, "#808080".to_owned(), (true, false)).collapse_x(),
            Self::DEFAULT_COLOURS
                .iter()
                .map(|colour| colour.to_string())
                .collect(),

            start_date.clone().unwrap(),
            start_date.map(|dt| dt.to_chrono().format("%d/%m/%Y").to_string())
                .unwrap_or(String::new()),
            end_date,
            "Early Day Motions (as supporting signatures) over date tabled (as time in days)",
        )
    }
}

impl From<&mut Vec<(EarlyDayMotion, Vec<Signature>)>> for SupportingSignaturesOverTimeGraph {
    fn from(value: &mut Vec<(EarlyDayMotion, Vec<Signature>)>) -> Self {
        Self::from(&*value)
    }
}

impl From<Vec<(EarlyDayMotion, Vec<Signature>)>> for SupportingSignaturesOverTimeGraph {
    fn from(value: Vec<(EarlyDayMotion, Vec<Signature>)>) -> Self {
        Self::from(&value)
    }
}

impl From<&SupportingSignaturesOverTimeGraph> for Document {
    fn from(value: &SupportingSignaturesOverTimeGraph) -> Self {
        let mut document = Document::new();

        document = document.add(
            Document::new()
                .add(
                    Text::new(value.title.clone())
                        .set("x", "50%")
                        .set("y", "50%")
                        .set("fill", "#707070")
                        .set("font-family", "sans-serif")
                        .set("font-size", "30px")
                        .set("font-weight", "bold")
                        .set("text-anchor", "middle")
                        .set("dominant-baseline", "middle"),
                )
                .set("width", "100%")
                .set("height", "5%"),
        );

        let ordered_x = value.graph.clone().collapse_x().points.into_iter();
        let mut ordered_y = value
            .graph
            .clone()
            .collapse_x()
            .points
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1));
        let highlighted = value.colours.iter().map(|colour| {
            let (x, y) = ordered_y.next().unwrap();

            let before = ordered_x
                .clone()
                .filter(|(x2, ..)| *x2 < x)
                .rev()
                .next()
                .unwrap_or((x, y));
            let after = ordered_x
                .clone()
                .filter(|(x2, ..)| *x2 > x)
                .next()
                .unwrap_or((x, y));

            let data = Data::new().move_to(before).line_to((x, y)).line_to(after);

            let path = Path::new()
                .set("fill", "none")
                .set("stroke", colour.as_str())
                .set("stroke-width", 5)
                .set("d", data);

            path
        });

        let mut graph = Document::from(&value.graph);

        for highlight in highlighted {
            graph = graph.add(highlight);
        }

        document = document.add(
            Document::new()
                .add(Group::from(Inverted::new(
                    graph
                        .set("preserveAspectRatio", "none")
                        .set("stroke-width", 4),
                    false,
                    true,
                    true,
                )))
                .set("y", "5%")
                .set("width", "100%")
                .set("height", "87.5%"),
        );

        document = document.add(
            Document::from(GraphYAxisLines::from(&value.graph))
                .set("y", "5%")
                .set("width", "100.1%")
                .set("height", "87.5%")
                .set("preserveAspectRatio", "none"),
        );

        let mut dashes = GraphXAxisDashes::from(&value.graph);

        let mut days = 0;
        let mut date = value.start_dt.to_chrono();
        while date.month() != 1 {
            days += date.day();
            date = date.checked_sub_months(Months::new(1)).unwrap();
        }
        days += date.day();

        dashes.interval = (365 - days as usize, 365);
        document = document.add(
            Document::from(dashes)
                .set("y", "92.5%")
                .set("width", "100%")
                .set("height", "2.5%")
                .set("preserveAspectRatio", "none"),
        );

        document = document.add(
            Document::new()
                .add(
                    Text::new(value.start_date.as_str())
                        .set("x", "0.75%")
                        .set("y", "62.5%")
                        .set("fill", "#707070")
                        .set("font-family", "sans-serif")
                        .set("font-size", "28px")
                        .set("font-weight", "bold")
                        .set("text-anchor", "start")
                        .set("dominant-baseline", "middle"),
                )
                .add(
                    Text::new(value.end_date.as_str())
                        .set("x", "99.25%")
                        .set("y", "62.5%")
                        .set("fill", "#707070")
                        .set("font-family", "sans-serif")
                        .set("font-size", "28px")
                        .set("font-weight", "bold")
                        .set("text-anchor", "end")
                        .set("dominant-baseline", "middle"),
                )
                .set("y", "95%")
                .set("width", "100%")
                .set("height", "5%")
                .set("preserveAspectRatio", "none"),
        );

        document = Document::new().add(
            document
                .set("x", "2.5%")
                .set("width", "97.5%")
                .set("height", "100%"),
        );

        let mut axis = GraphYAxisNumbers::from(&value.graph);
        axis.font_size = axis.height / 23;
        document = document.add(
            Group::new()
                .add(
                    Document::from(axis)
                        .set("x", "-0.9%")
                        .set("y", "5%")
                        .set("width", "2%")
                        .set("height", "87.5%")
                        .set("preserveAspectRatio", "none"),
                )
                .set("transform", "scale(12.5, 1)"),
        );

        document = Document::new().add(document.set("width", "100%").set("height", "90%"));

        let labels = value
            .colours
            .iter()
            .enumerate()
            .map(|(i, colour)| {
                (
                    colour.clone(),
                    "#".to_owned() + (i + 1).to_string().as_str(),
                )
            })
            .collect::<Vec<_>>();

        document = document.add(
            Document::new()
                .add(Group::from(StackedGraphLabels::from(labels)))
                .set("width", "100%")
                .set("height", "10%")
                .set("y", "90%"),
        );

        document
    }
}

impl From<SupportingSignaturesOverTimeGraph> for Document {
    fn from(value: SupportingSignaturesOverTimeGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut SupportingSignaturesOverTimeGraph> for Document {
    fn from(value: &mut SupportingSignaturesOverTimeGraph) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackedSignaturesPerPartyOverTimeGraph {
    pub graph: StackedGraph,
    pub title: String,
    pub start_date: DateTime,
    pub end_date: DateTime,
    pub labels: Vec<(String, String)>,
}

impl StackedSignaturesPerPartyOverTimeGraph {
    pub const TIME_DIVISOR: usize = 1000 * 60 * 60;

    pub fn new(
        graph: StackedGraph,
        title: impl Into<String>,
        start_date: DateTime,
        end_date: DateTime,
        labels: Vec<(String, String)>,
    ) -> Self {
        Self {
            graph,
            title: title.into(),
            start_date,
            end_date,
            labels,
        }
    }
}

impl From<&Vec<Signature>> for StackedSignaturesPerPartyOverTimeGraph {
    fn from(value: &Vec<Signature>) -> Self {
        let min_x = value
            .iter()
            .map(|signature| signature.creation_date)
            .sorted()
            .next()
            .map(DateTime::timestamp_millis)
            .unwrap_or(0) as usize;

        let parties = value
            .iter()
            .unique_by(|signature| signature.party._id)
            .map(|signature| &signature.party);

        let party_signatures = parties.map(|party| {
            (
                party,
                value
                    .iter()
                    .filter(|signature| signature.party._id == party._id)
                    .sorted_by(|a, b| (**a).sponsoring_order.cmp(&(**b).sponsoring_order))
                    .sorted_by(|a, b| (**a).creation_date.cmp(&(**b).creation_date))
                    .collect::<Vec<_>>(),
            )
        });

        let graphs = party_signatures
            .map(|(party, signatures)| {
                LineGraph::new(
                    signatures
                        .into_iter()
                        .enumerate()
                        .map(|(y, signature)| {
                            (
                                (signature.creation_date.timestamp_millis() as usize - min_x)
                                    / Self::TIME_DIVISOR,
                                y + 1,
                            )
                        })
                        .collect(),
                    "#".to_owned() + &party.background_colour,
                    (true, true),
                )
            })
            .sorted_by(|a, b| b.max_y().cmp(&a.max_y()))
            .collect();

        let labels = value
            .iter()
            .unique_by(|signature| signature.party._id)
            .map(|signature| {
                (
                    signature,
                    value
                        .iter()
                        .filter(|s| s.party._id == signature.party._id)
                        .map(|_| 1)
                        .sum::<usize>(),
                )
            })
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .map(|(signature, ..)| {
                (
                    "#".to_owned() + signature.party.background_colour.as_str(),
                    signature.party.abbreviation.clone(),
                )
            })
            .collect::<Vec<_>>();

        let start_date = value
            .iter()
            .sorted_by(|a, b| a.sponsoring_order.cmp(&b.sponsoring_order))
            .sorted_by(|a, b| a.creation_date.cmp(&b.creation_date))
            .next()
            .unwrap()
            .creation_date;

        let end_date = value
            .iter()
            .sorted_by(|a, b| b.sponsoring_order.cmp(&a.sponsoring_order))
            .sorted_by(|a, b| b.creation_date.cmp(&a.creation_date))
            .next()
            .unwrap()
            .creation_date;

        Self::new(
            StackedGraph::new(graphs).fill_gaps(),
            "Supporting signatures (per party) over date signed (as time in days)",
            start_date,
            end_date,
            labels,
        )
    }
}

impl From<Vec<Signature>> for StackedSignaturesPerPartyOverTimeGraph {
    fn from(value: Vec<Signature>) -> Self {
        Self::from(&value)
    }
}

impl From<&mut Vec<Signature>> for StackedSignaturesPerPartyOverTimeGraph {
    fn from(value: &mut Vec<Signature>) -> Self {
        Self::from(&*value)
    }
}

impl From<&StackedSignaturesPerPartyOverTimeGraph> for Document {
    fn from(value: &StackedSignaturesPerPartyOverTimeGraph) -> Self {
        let mut document = Document::new();

        document = document.add(
            Document::new()
                .add(
                    Text::new(value.title.clone())
                        .set("x", "50%")
                        .set("y", "50%")
                        .set("fill", "#707070")
                        .set("font-family", "sans-serif")
                        .set("font-size", "30px")
                        .set("font-weight", "bold")
                        .set("text-anchor", "middle")
                        .set("dominant-baseline", "middle"),
                )
                .set("width", "100%")
                .set("height", "5%"),
        );

        document = document.add(
            Document::new()
                .add(Group::from(Inverted::new(
                    Document::from(&value.graph).set("preserveAspectRatio", "none"),
                    false,
                    true,
                    true,
                )))
                .set("y", "5%")
                .set("width", "100%")
                .set("height", "87.5%"),
        );

        document = document.add(
            Document::from(GraphYAxisLines::from(&value.graph))
                .set("x", "2%")
                .set("y", "5%")
                .set("width", "98.1%")
                .set("height", "87.5%")
                .set("preserveAspectRatio", "none"),
        );

        let mut axis = GraphYAxisNumbers::from(&value.graph);
        axis.font_size = axis.height / 23;
        document = document.add(
            Group::new()
                .add(
                    Document::from(axis)
                        .set("x", "-0.875%")
                        .set("y", "5%")
                        .set("width", "2%")
                        .set("height", "87.5%")
                        .set("preserveAspectRatio", "none"),
                )
                .set("transform", "scale(12.5, 1)"),
        );

        let mut dashes = GraphXAxisDashes::from(&value.graph);
        dashes.interval = (23 - value.start_date.to_chrono().hour() as usize, 24);
        document = document.add(
            Document::from(dashes)
                .set("y", "92.5%")
                .set("width", "100%")
                .set("height", "2.5%")
                .set("preserveAspectRatio", "none"),
        );

        document = document.add(
            Document::new()
                .add(
                    Text::new(value.start_date.to_chrono().format("%d/%m/%Y").to_string())
                        .set("x", "0.75%")
                        .set("y", "62.5%")
                        .set("fill", "#707070")
                        .set("font-family", "sans-serif")
                        .set("font-size", "28px")
                        .set("font-weight", "bold")
                        .set("text-anchor", "start")
                        .set("dominant-baseline", "middle"),
                )
                .add(
                    Text::new(value.end_date.to_chrono().format("%d/%m/%Y").to_string())
                        .set("x", "99.25%")
                        .set("y", "62.5%")
                        .set("fill", "#707070")
                        .set("font-family", "sans-serif")
                        .set("font-size", "28px")
                        .set("font-weight", "bold")
                        .set("text-anchor", "end")
                        .set("dominant-baseline", "middle"),
                )
                .set("y", "95%")
                .set("width", "100%")
                .set("height", "5%")
                .set("preserveAspectRatio", "none"),
        );

        document = Document::new().add(document.set("width", "100%").set("height", "90%"));

        document = document.add(
            Document::new()
                .add(Group::from(StackedGraphLabels::from(value.labels.clone())))
                .set("width", "100%")
                .set("height", "10%")
                .set("y", "90%"),
        );

        document
    }
}

impl From<StackedSignaturesPerPartyOverTimeGraph> for Document {
    fn from(value: StackedSignaturesPerPartyOverTimeGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut StackedSignaturesPerPartyOverTimeGraph> for Document {
    fn from(value: &mut StackedSignaturesPerPartyOverTimeGraph) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackedGraph {
    pub graphs: Vec<LineGraph>,
}

impl StackedGraph {
    pub const fn new(graphs: Vec<LineGraph>) -> Self {
        Self { graphs }
    }

    pub fn fill_gaps(mut self) -> Self {
        let xs = 0..=self.max_x();

        let filled = self.graphs.iter().map(|graph| {
            let gaps = xs
                .clone()
                .filter(|x| graph.points.iter().find(|(x2, ..)| x2 == x).is_none())
                .map(|x| {
                    (
                        x,
                        graph
                            .points
                            .iter()
                            .filter(|(x2, ..)| *x2 < x)
                            .rev()
                            .next()
                            .map(|(.., y2)| *y2)
                            .unwrap_or(0),
                    )
                });

            let mut graph = graph.clone();
            graph.points.extend(gaps);
            graph.points = graph
                .points
                .into_iter()
                .sorted_by(|a, b| a.0.cmp(&b.0))
                .collect();

            graph
        });

        self.graphs = filled.collect();

        self
    }

    pub fn stack_y(&self, i: usize, x: usize) -> usize {
        self.graphs
            .iter()
            .enumerate()
            .filter(|(index, ..)| i > *index)
            .map(|(.., graph)| {
                graph
                    .points
                    .iter()
                    .filter(|(x2, ..)| x >= *x2)
                    .rev()
                    .next()
                    .map(|(.., y2)| *y2)
                    .unwrap_or(0)
            })
            .sum()
    }

    pub fn stack_graph(&self, i: usize, graph: &LineGraph) -> impl Iterator<Item = (usize, usize)> {
        graph
            .points
            .iter()
            .map(move |(x, y)| (*x, self.stack_y(i, *x) + y))
    }

    pub fn max_x(&self) -> usize {
        self.graphs
            .iter()
            .map(|graph| graph.max_x())
            .max()
            .unwrap_or(0)
    }

    pub fn max_y(&self) -> usize {
        self.graphs.iter().map(|graph| graph.max_y()).sum()
    }
}

impl From<&StackedGraph> for Document {
    fn from(value: &StackedGraph) -> Self {
        Document::new().add(Group::from(value)).set(
            "viewBox",
            format!("-8 -8 {} {}", value.max_x() + 16, value.max_y() + 16),
        )
    }
}

impl From<StackedGraph> for Document {
    fn from(value: StackedGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut StackedGraph> for Document {
    fn from(value: &mut StackedGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<&StackedGraph> for Group {
    fn from(value: &StackedGraph) -> Self {
        let mut group = Group::new();

        for (.., graph, stacked) in value
            .graphs
            .iter()
            .enumerate()
            .map(|(index, graph)| {
                (
                    index,
                    graph,
                    value
                        .stack_graph(index, graph)
                        .sorted_by(|a, b| b.1.cmp(&a.1))
                        .unique_by(|(x, ..)| *x)
                        .sorted_by(|a, b| a.0.cmp(&b.0))
                        .collect::<Vec<_>>(),
                )
            })
            .rev()
        {
            let graph = LineGraph::new(stacked, &graph.colour, graph.fill);

            group = group.add(Path::from(graph));
        }

        group
    }
}

impl From<StackedGraph> for Group {
    fn from(value: StackedGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut StackedGraph> for Group {
    fn from(value: &mut StackedGraph) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineGraph {
    pub points: Vec<(usize, usize)>,
    pub colour: String,
    pub fill: (bool, bool),
}

impl LineGraph {
    pub fn new(points: Vec<(usize, usize)>, colour: impl Into<String>, fill: (bool, bool)) -> Self {
        Self {
            points,
            colour: colour.into(),
            fill,
        }
    }

    pub fn fill_gaps(mut self) -> Self {
        let xs = 0..=self.max_x();

        let gaps = xs
            .clone()
            .filter(|x| self.points.iter().find(|(x2, ..)| x2 == x).is_none())
            .map(|x| (x, 0))
            .collect::<Vec<_>>();

        self.points.extend(gaps);
        self.points = self
            .points
            .into_iter()
            .sorted_by(|a, b| a.0.cmp(&b.0))
            .collect();

        self
    }

    pub fn collapse_x(mut self) -> Self {
        self.points = self
            .points
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .sorted_by(|a, b| a.0.cmp(&b.0))
            .unique_by(|(x, ..)| *x)
            .collect();

        self
    }

    pub fn max_x(&self) -> usize {
        self.points.iter().map(|(x, ..)| *x).max().unwrap_or(0)
    }

    pub fn max_y(&self) -> usize {
        self.points.iter().map(|(.., y)| *y).max().unwrap_or(0)
    }
}

impl From<Vec<(usize, usize)>> for LineGraph {
    fn from(value: Vec<(usize, usize)>) -> Self {
        Self::new(value, DARK_GREY, (false, false))
    }
}

impl From<&LineGraph> for Document {
    fn from(value: &LineGraph) -> Self {
        Document::new().add(Path::from(value)).set(
            "viewBox",
            format!("-8 -8 {} {}", value.max_x() + 16, value.max_y() + 16),
        )
    }
}

impl From<LineGraph> for Document {
    fn from(value: LineGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut LineGraph> for Document {
    fn from(value: &mut LineGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<&LineGraph> for Path {
    fn from(value: &LineGraph) -> Self {
        let mut data = Data::new().move_to((0.0, 0.0));

        let ordered = value.points.iter().sorted_by(|a, b| a.0.cmp(&b.0));

        for (x, y) in ordered.clone() {
            data = data.line_to((*x, *y));
        }

        if value.fill.0
            && let Some((_, y)) = ordered.rev().next()
        {
            data = data.line_by((0, *y as isize * -1));
            data = data.line_to((0, 0));
        }

        Path::new()
            .set(
                "fill",
                if value.fill.1 {
                    value.colour.as_str()
                } else {
                    "none"
                },
            )
            .set("stroke", value.colour.clone())
            .set("d", data)
    }
}

impl From<LineGraph> for Path {
    fn from(value: LineGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut LineGraph> for Path {
    fn from(value: &mut LineGraph) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BarGraph {
    pub points: Vec<(usize, usize)>,
    pub colour: String,
}

impl BarGraph {
    pub fn new(points: Vec<(usize, usize)>, colour: impl Into<String>) -> Self {
        Self {
            points,
            colour: colour.into(),
        }
    }

    pub fn collapse_x(mut self) -> Self {
        self.points = self
            .points
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .sorted_by(|a, b| a.0.cmp(&b.0))
            .unique_by(|(x, ..)| *x)
            .collect();

        self
    }

    pub fn max_x(&self) -> usize {
        self.points.iter().map(|(x, ..)| *x).max().unwrap_or(0)
    }

    pub fn max_y(&self) -> usize {
        self.points.iter().map(|(.., y)| *y).max().unwrap_or(0)
    }
}

impl From<Vec<(usize, usize)>> for BarGraph {
    fn from(value: Vec<(usize, usize)>) -> Self {
        Self::new(value, DARK_GREY)
    }
}

impl From<&BarGraph> for Document {
    fn from(value: &BarGraph) -> Self {
        Document::new().add(Group::from(value)).set(
            "viewBox",
            format!("-8 -8 {} {}", value.max_x() + 16, value.max_y() + 16),
        )
    }
}

impl From<BarGraph> for Document {
    fn from(value: BarGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut BarGraph> for Document {
    fn from(value: &mut BarGraph) -> Self {
        Self::from(&*value)
    }
}

impl From<&BarGraph> for Group {
    fn from(value: &BarGraph) -> Self {
        let mut group = Group::new();

        let ordered = value.points.iter().sorted_by(|a, b| a.0.cmp(&b.0));

        for (x, y) in ordered.clone() {
            group = group.add(
                Rectangle::new()
                    .set("fill", value.colour.as_str())
                    .set("x", *x)
                    .set("width", 1)
                    .set("height", *y),
            );
        }

        group
    }
}

impl From<BarGraph> for Group {
    fn from(value: BarGraph) -> Self {
        Self::from(&value)
    }
}

impl From<&mut BarGraph> for Group {
    fn from(value: &mut BarGraph) -> Self {
        Self::from(&*value)
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Inverted<T: Into<Box<dyn Node>>> {
    pub node: T,
    pub x: bool,
    pub y: bool,
    pub centered: bool,
}

impl<T: Into<Box<dyn Node>>> Inverted<T> {
    pub const fn new(node: T, x: bool, y: bool, centered: bool) -> Self {
        Self {
            node,
            x,
            y,
            centered,
        }
    }
}

impl<T: Into<Box<dyn Node>>> From<Inverted<T>> for Group {
    fn from(value: Inverted<T>) -> Self {
        let mut group = Group::new().add(value.node);

        match (value.x, value.y) {
            (true, true) => group = group.set("transform", "scale(-1 -1)"),
            (true, false) => group = group.set("transform", "scale(-1 1)"),
            (false, true) => group = group.set("transform", "scale(1 -1)"),
            (false, false) => group = group.set("transform", "scale(1 1)"),
        }

        if value.centered {
            group = group.set("transform-origin", "center");
        }

        group
    }
}
