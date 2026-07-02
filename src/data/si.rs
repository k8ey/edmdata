use bson::DateTime;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    api::si::{PublishedBusinessItem, PublishedStatutoryInstrument},
    data::House,
};

pub const SI_COLLECTION: &str = "statutory-instruments";

pub type StatutoryInstrumentID = String;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatutoryInstrumentExtra {}

impl From<&PublishedStatutoryInstrument> for StatutoryInstrumentExtra {
    fn from(_value: &PublishedStatutoryInstrument) -> Self {
        Self {}
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatutoryInstrument {
    pub _id: StatutoryInstrumentID,
    pub name: String,
    pub procedure: Procedure,
    pub laying_body: LayingBody,
    pub enabling_acts: Vec<String>,
    pub commons_laying_date: Option<DateTime>,
    pub lords_laying_date: Option<DateTime>,
    pub followed_by: Option<StatutoryInstrumentID>,
    pub preceded_by: Option<StatutoryInstrumentID>,
    pub business_items: Vec<BusinessItem>,
    pub paper: StatutoryInstrumentPaper,
    pub link: String,

    pub _updated: DateTime,
}

impl From<PublishedStatutoryInstrument> for StatutoryInstrument {
    fn from(value: PublishedStatutoryInstrument) -> Self {
        Self {
            _id: value.statutory_instrument.statutory_instrument.id,
            name: value.statutory_instrument.statutory_instrument.name,
            procedure: Procedure {
                _id: value.statutory_instrument.statutory_instrument.procedure.id,
                name: value
                    .statutory_instrument
                    .statutory_instrument
                    .procedure
                    .name,
            },
            laying_body: LayingBody {
                _id: value.laying_body.id,
                name: value.laying_body.name,
                department_id: value
                    .laying_body
                    .department_id
                    .map(|department_id| department_id as u32),
            },
            enabling_acts: value.enabling_acts.into_iter().map(|act| act.id).collect(),
            followed_by: value.followed_by_instrument.map(|si| si.id),
            preceded_by: value.preceded_by_instrument.map(|si| si.id),
            business_items: value
                .business_items
                .unwrap_or(Vec::new())
                .into_iter()
                .map(|bi| BusinessItem {
                    _id: bi.id,
                    steps: bi.steps,
                    item_date: bi.item_date.map(|dt| dt.and_utc().into()),
                    link: bi.link.unwrap_or(String::new()),
                    sequence: bi.sequence,
                    houses: bi
                        .houses
                        .into_iter()
                        .map(|house| (house as u8).into())
                        .collect(),
                })
                .collect(),
            paper: StatutoryInstrumentPaper {
                paper_prefix: value
                    .statutory_instrument
                    .paper_prefix
                    .unwrap_or(String::new()),
                paper_number: value.statutory_instrument.paper_number,
                paper_year: value
                    .statutory_instrument
                    .paper_year
                    .map(|year| year.parse().ok())
                    .flatten(),
                paper_made_date: value
                    .statutory_instrument
                    .paper_made_date
                    .map(|dt| dt.and_utc().into()),
                paper_coming_into_force_date: value
                    .paper_coming_into_force_date
                    .map(|dt| dt.and_utc().into()),
                paper_coming_into_force_note: value
                    .paper_coming_into_force_note
                    .unwrap_or(String::new()),
            },
            commons_laying_date: Some(
                value
                    .statutory_instrument
                    .commons_laying_date
                    .and_utc()
                    .into(),
            ),
            lords_laying_date: value
                .statutory_instrument
                .lords_laying_date
                .map(|dt| dt.and_utc().into()),
            link: value.link.unwrap_or(String::new()),
            _updated: DateTime::now(),
        }
    }
}

pub type ProcedureID = String;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Procedure {
    pub _id: ProcedureID,
    pub name: String,
}

pub type LayingBodyID = String;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayingBody {
    pub _id: LayingBodyID,
    pub name: String,
    pub department_id: Option<u32>,
}

pub type BusinessItemID = String;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BusinessItem {
    pub _id: BusinessItemID,
    pub steps: Vec<String>,
    pub item_date: Option<DateTime>,
    pub link: String,
    pub sequence: Option<i32>,
    pub houses: Vec<House>,
}

impl From<PublishedBusinessItem> for BusinessItem {
    fn from(value: PublishedBusinessItem) -> Self {
        Self {
            _id: value.id,
            steps: value.steps,
            item_date: value.item_date.map(|dt| dt.and_utc().into()),
            link: value.link.unwrap_or(String::new()),
            sequence: value.sequence,
            houses: value
                .houses
                .into_iter()
                .map(|house| (house as u8).into())
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatutoryInstrumentPaper {
    pub paper_prefix: String,
    pub paper_number: Option<i32>,
    pub paper_year: Option<i32>,
    pub paper_made_date: Option<DateTime>,
    pub paper_coming_into_force_date: Option<DateTime>,
    pub paper_coming_into_force_note: String,
}
