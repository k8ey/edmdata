use std::{error::Error, path::Path};

use edmdata::{
    data::{Dataset, edm::MotionKind},
    query::*,
    svg::{
        StackedSignaturesPerPartyOverTimeGraph, SupportingSignaturesOverTimeGraph,
        WIDTH_HEIGHT_BASE, WIDTH_HEIGHT_RATIO,
    },
};
use futures_util::TryStreamExt;
use svg::{Document, Node};

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt().init();
    let dataset = Dataset::new().await;
    dataset.run_refresh_tasks().await.unwrap();

    edm240_stats(&dataset).await.unwrap();
}

pub async fn edm240_stats(dataset: &Dataset) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("data/edm240/data")?;
    std::fs::create_dir_all("data/edm240/graphs")?;

    let items = dataset
        .edms
        .aggregate(
            MainQuery::new(
                vec![bson::doc! {
                    "$match": {
                        "item._id": 65938,
                    },
                }]
                .doc_add(CollectSignatures::new()),
            )
            .build(),
        )
        .await?
        .with_type::<QueryItem>()
        .try_collect::<Vec<_>>()
        .await?;

    MainQuery::write_json(&items, "data/edm240/data/edm240.json")?;

    let mut graph =
        StackedSignaturesPerPartyOverTimeGraph::from(items.first().unwrap().signatures.clone());
    graph.title = "EDM 240: ".to_owned() + &graph.title;

    MainGraph::write_svg(
        Document::from(graph),
        "data/edm240/graphs/edm240-stacked-per-party.svg",
    )?;

    for i in 1..=3 {
        let filters = Vec::<bson::Document>::new()
            .doc_add(FilterByMotionKind::new(vec![
                MotionKind::Fatal,
                MotionKind::SeemsFatal,
            ]))
            .doc_add(FilterByDate::new_by_year(2026 - 10 * i, 2027))
            .doc_add(CollectSignatures::new());

        let items = dataset
            .edms
            .aggregate(MainQuery::new(filters).build())
            .await?
            .with_type::<QueryItem>()
            .try_collect::<Vec<_>>()
            .await?;

        MainQuery::write_json(
            &items,
            format!(
                "data/edm240/data/fatal-prayer-motions-last-{}-years.json",
                10 * i
            ),
        )?;

        let mut graph = SupportingSignaturesOverTimeGraph::from(
            items
                .iter()
                .map(|item| (item.item.item.clone(), item.signatures.clone()))
                .collect::<Vec<_>>(),
        );
        graph.title =
            "Fatal Prayer Motions (as supporting signatures) over date tabled (as time in days)"
                .to_owned();

        MainGraph::write_svg(
            Document::from(graph),
            format!(
                "data/edm240/graphs/fatal-prayer-motions-last-{}-years.svg",
                10 * i
            ),
        )?;
    }

    let filters = Vec::<bson::Document>::new()
        .doc_add(FilterByMotionKind::new(vec![
            MotionKind::Fatal,
            MotionKind::SeemsFatal,
        ]))
        .doc_add(CollectSignatures::new());

    let items = dataset
        .edms
        .aggregate(MainQuery::new(filters).build())
        .await?
        .with_type::<QueryItem>()
        .try_collect::<Vec<_>>()
        .await?;

    MainQuery::write_json(&items, "data/edm240/data/fatal-prayer-motions-all.json")?;

    let mut graph = SupportingSignaturesOverTimeGraph::from(
        items
            .iter()
            .map(|item| (item.item.item.clone(), item.signatures.clone()))
            .collect::<Vec<_>>(),
    );
    graph.title = "Fatal Prayer Motions (as supporting signatures) over date tabled (as time in days)".to_owned();

    MainGraph::write_svg(
        Document::from(graph),
        format!("data/edm240/graphs/fatal-prayer-motions-all.svg"),
    )?;

    for i in 1..=2 {
        let filters = Vec::<bson::Document>::new()
            .doc_add(FilterByDate::new_by_year(2026 - 5 * i, 2027))
            .doc_add(CollectSignatures::new());

        let items = dataset
            .edms
            .aggregate(MainQuery::new(filters).build())
            .await?
            .with_type::<QueryItem>()
            .try_collect::<Vec<_>>()
            .await?;

        MainQuery::write_json(
            &items,
            format!("data/edm240/data/edms-last-{}-years.json", 5 * i),
        )?;

        MainGraph::write_svg(
            Document::from(SupportingSignaturesOverTimeGraph::from(
                items
                    .iter()
                    .map(|item| (item.item.item.clone(), item.signatures.clone()))
                    .collect::<Vec<_>>(),
            )),
            format!("data/edm240/graphs/edms-last-{}-years.svg", 5 * i),
        )?;
    }

    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct MainQuery {
    pub filters: Vec<bson::Document>,
}

impl Query for MainQuery {
    fn build(&self) -> Vec<bson::Document> {
        let mut pipeline = self.filters.clone();

        pipeline.extend(Self::group(true));
        pipeline.extend(Self::group(false));

        pipeline = pipeline
            .doc_add(FilterSignaturesBySupporting::new(true))
            .doc_add(SetEarlyDayMotionData::new())
            .doc_add(SortBySignatureCount::new(false));

        pipeline
    }
}

impl MainQuery {
    pub const fn new(filters: Vec<bson::Document>) -> Self {
        Self { filters }
    }

    pub fn group(supporting: bool) -> Vec<bson::Document> {
        let root = if supporting {
            "supporting"
        } else {
            "withdrawn"
        };

        let mut query = GroupQuery::default()
            .prepend(FilterSignaturesBySupporting::new(supporting))
            .append(ResetSignatures::new())
            .with(SetSignatureCount::new(format!("{root}.total")))
            .with(
                FilterSignaturesByBench::new(true, None)
                    .doc_add(SetSignatureCount::new(format!("{root}.backbench.total"))),
            )
            .with(
                FilterSignaturesByBench::new(false, None)
                    .doc_add(SetSignatureCount::new(format!("{root}.frontbench.total"))),
            )
            .with(
                FilterSignaturesByGender::new(vec!["F".to_owned()])
                    .doc_add(SetSignatureCount::new(format!("{root}.women.total"))),
            )
            .with(
                FilterSignaturesByGender::new(vec!["M".to_owned()])
                    .doc_add(SetSignatureCount::new(format!("{root}.men.total"))),
            )
            .with(SetSignaturesByParty::new(format!("{root}.by_party")))
            .with(SetSignaturesByDate::new(
                format!("{root}.by_date.creation"),
                true,
            ));

        if root == "withdrawn".to_owned() {
            query = query.with(SetSignaturesByDate::new(
                format!("{root}.by_date.withdrawal"),
                false,
            ));
        }

        query
            .with(SetSignaturesByMembership::new(
                format!("{root}.by_membership.initial"),
                true,
            ))
            .with(SetSignaturesByMembership::new(
                format!("{root}.by_membership.latest"),
                false,
            ))
            .build()
    }

    pub fn write_json(
        items: &Vec<QueryItem>,
        path: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        std::fs::write(
            path,
            format!(
                "[{}]",
                items
                    .clone()
                    .iter_mut()
                    .take(10)
                    .map(|edm| edm.result.as_document_mut().unwrap())
                    .map(|edm| {
                        edm.insert(
                            "date_tabled",
                            edm.get_datetime("date_tabled")
                                .unwrap()
                                .to_chrono()
                                .format("%d/%m/%Y")
                                .to_string(),
                        );

                        edm
                    })
                    .enumerate()
                    .map(|(x, edm)| {
                        edm.insert("ranking", x as i32 + 1);

                        edm
                    })
                    .map(|edm| format!("{edm:#}"))
                    .collect::<Vec<_>>()
                    .join(",\n"),
            )
            .as_bytes(),
        )
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MainGraph;

impl MainGraph {
    pub fn write_svg(
        node: impl Into<Box<dyn Node>>,
        path: impl AsRef<Path> + Clone,
    ) -> Result<(), Box<dyn Error>> {
        let (width, height) = (
            WIDTH_HEIGHT_BASE * WIDTH_HEIGHT_RATIO.0,
            WIDTH_HEIGHT_BASE * WIDTH_HEIGHT_RATIO.1,
        );
        svg::save(
            path.clone(),
            &Document::new()
                .set("width", width)
                .set("height", height)
                .add(node),
        )
        .unwrap();

        let svg = std::fs::read_to_string(path.clone())?;
        let mut options = resvg::usvg::Options::default();
        options.dpi = 96.0 * 96.0;
        options.fontdb_mut().load_system_fonts();
        let tree = resvg::usvg::Tree::from_str(svg.as_str(), &options).unwrap();
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(tree.size().width() as u32, tree.size().height() as u32)
                .unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );

        pixmap.save_png(path.as_ref().with_extension("png"))?;

        Ok(())
    }
}
