use std::{error::Error, path::Path};

use edmdata::{
    data::{Dataset, edm::MotionKind},
    query::*,
    svg::{StackedSignaturesPerPartyOverTimeGraph, SupportingSignaturesOverTimeGraph, save_node},
};
use futures_util::TryStreamExt;
use svg::Document;

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt().init();
    let dataset = Dataset::new().await;
    dataset.run_refresh_tasks().await.unwrap();

    std::fs::create_dir_all("data/edm240/data").unwrap();
    std::fs::create_dir_all("data/edm240/graphs").unwrap();

    edm240_stats(&dataset).await.unwrap();

    for i in 1..=2 {
        early_day_motions_stats(
            &dataset,
            i * 5,
            format!("fatal-prayer-motions-last-{}-years", i * 5),
        )
        .await
        .unwrap();
    }

    fatal_prayer_motions_stats(&dataset, 40, "fatal-prayer-motions-all")
        .await
        .unwrap();

    for i in 1..=3 {
        fatal_prayer_motions_stats(
            &dataset,
            i * 10,
            format!("fatal-prayer-motions-last-{}-years", i * 10),
        )
        .await
        .unwrap();
    }
}

pub async fn edm240_stats(dataset: &Dataset) -> Result<(), Box<dyn Error>> {
    let items = dataset
        .edms
        .aggregate(
            StatsQuery::new(
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

    println!(
        "Total supporting signatures: <b>{}</b><br>",
        items[0].signatures.len()
    );
    println!();

    println!("Supporting signatures by bench:");
    println!("<ul>");
    for bench in [("backbench", "Backbench"), ("frontbench", "Frontbench")] {
        let total = items[0]
            .result
            .as_document()
            .unwrap()
            .get_document("supporting")
            .unwrap()
            .get_document(bench.0)
            .unwrap()
            .get_i32("total")
            .unwrap();

        println!("<li>");
        println!(
            "    {}: <b>{}</b> (<u>{:.1}%</u> of signatures)",
            bench.1,
            total,
            total as f64 / items[0].signatures.len() as f64 * 100.0
        );
        println!("</li>");
    }
    println!("</ul>");
    println!();

    println!("Supporting signatures by party:");
    println!("<ul>");
    for party in items[0]
        .result
        .as_document()
        .unwrap()
        .get_document("supporting")
        .unwrap()
        .get_document("by_party")
        .unwrap()
        .into_iter()
    {
        let total = party.1.as_i32().unwrap();

        println!("<li>");
        println!(
            "    {}: <b>{}</b> (<u>{:.1}%</u> of signatures)",
            party.0,
            total,
            total as f64 / items[0].signatures.len() as f64 * 100.0
        );
        println!("</li>");
    }
    println!("</ul>");
    println!();

    StatsQuery::save_json(&items, "data/edm240/data/edm240.json")?;

    let mut graph =
        StackedSignaturesPerPartyOverTimeGraph::from(items.first().unwrap().signatures.clone());
    graph.title = "EDM 240: ".to_owned() + &graph.title;

    save_node(
        Document::from(graph),
        "data/edm240/graphs/edm240-stacked-per-party.svg",
    )?;

    Ok(())
}

pub async fn early_day_motions_stats(
    dataset: &Dataset,
    years: i32,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let filters = Vec::<bson::Document>::new()
        .doc_add(FilterByDate::new_by_year(2026 - years, 2027))
        .doc_add(CollectSignatures::new());

    let items = dataset
        .edms
        .aggregate(StatsQuery::new(filters).build())
        .await?
        .with_type::<QueryItem>()
        .try_collect::<Vec<_>>()
        .await?;

    StatsQuery::save_json(
        &items,
        Path::new("data/edm240/data/data.json")
            .with_file_name(path.as_ref())
            .with_extension("json"),
    )?;

    save_node(
        Document::from(SupportingSignaturesOverTimeGraph::from(
            items
                .iter()
                .map(|item| (item.item.item.clone(), item.signatures.clone()))
                .collect::<Vec<_>>(),
        )),
        Path::new("data/edm240/graphs/graph.svg")
            .with_file_name(path.as_ref())
            .with_extension("svg"),
    )?;

    Ok(())
}

pub async fn fatal_prayer_motions_stats(
    dataset: &Dataset,
    years: i32,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let filters = Vec::<bson::Document>::new()
        .doc_add(FilterByMotionKind::new(vec![
            MotionKind::Fatal,
            MotionKind::SeemsFatal,
        ]))
        .doc_add(FilterByDate::new_by_year(2026 - years, 2027))
        .doc_add(CollectSignatures::new());

    let items = dataset
        .edms
        .aggregate(StatsQuery::new(filters).build())
        .await?
        .with_type::<QueryItem>()
        .try_collect::<Vec<_>>()
        .await?;

    StatsQuery::save_json(
        &items,
        Path::new("data/edm240/data/data.json")
            .with_file_name(path.as_ref())
            .with_extension("json"),
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

    save_node(
        Document::from(graph),
        Path::new("data/edm240/graphs/graph.svg")
            .with_file_name(path.as_ref())
            .with_extension("svg"),
    )?;

    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct StatsQuery {
    pub filters: Vec<bson::Document>,
}

impl Query for StatsQuery {
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

impl StatsQuery {
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

    pub fn save_json(items: &Vec<QueryItem>, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
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
