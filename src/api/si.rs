use bson::DateTime;
use chrono::{NaiveDateTime, Utc};
use derive_more::{Display, Error, From};
use mongodb::bson::doc;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiError, ApiRequest},
    data::{Dataset, si::StatutoryInstrument},
};

pub const STATUTORY_INSTRUMENT_API_HOST: &str = "https://statutoryinstruments-api.parliament.uk";
pub const STATUTORY_INSTRUMENT_PARTIAL_PATH: &str = "api/v2/StatutoryInstrument";

#[derive(Debug, Display, Error, From)]
pub enum PublishedStatutoryInstrumentDataError {
    #[display("SI ID has an invalid format: ({_0})")]
    #[from(skip)]
    InvalidID(#[error(not(source))] String),

    #[display("SI ({_0}) Procedure ID has an invalid format: ({_1})")]
    #[from(skip)]
    InvalidProcedureID(#[error(not(source))] String, #[error(not(source))] String),

    #[display("SI ({_0}) Followed SI ID has an invalid format: ({_1})")]
    #[from(skip)]
    InvalidFollowedStatutoryInstrumentID(
        #[error(not(source))] String,
        #[error(not(source))] String,
    ),

    #[display("SI ({_0}) Preceded SI ID has an invalid format: ({_1})")]
    #[from(skip)]
    InvalidPrecededStatutoryInstrumentID(
        #[error(not(source))] String,
        #[error(not(source))] String,
    ),

    #[display("SI ({_0}) LB ID has an invalid format: ({_1})")]
    #[from(skip)]
    InvalidLayingBodyID(#[error(not(source))] String, #[error(not(source))] String),

    #[display("SI ({_0}) LB Department ID was > 1: ({_1})")]
    #[from(skip)]
    InvalidLayingBodyDepartmentID(#[error(not(source))] String, #[error(not(source))] i32),

    #[display("SI ({_0}) AOP ID has an invalid format: ({_1})")]
    #[from(skip)]
    InvalidActOfParliamentID(#[error(not(source))] String, #[error(not(source))] String),

    #[display("SI ({_0}) BI ID has an invalid format: ({_1})")]
    #[from(skip)]
    InvalidBusinessItemID(#[error(not(source))] String, #[error(not(source))] String),

    MongoError(mongodb::error::Error),

    ApiError(ApiError),
}

pub struct RequestStatutoryInstrument {
    pub statutory_instrument_id: String,
}

impl RequestStatutoryInstrument {
    pub fn new(statutory_instrument_id: impl Into<String>) -> Self {
        Self {
            statutory_instrument_id: statutory_instrument_id.into(),
        }
    }
}

impl ApiRequest for RequestStatutoryInstrument {
    type Response = PublishedStatutoryInstrument;

    fn url(&self) -> impl Into<String> {
        format!(
            "{STATUTORY_INSTRUMENT_API_HOST}/{STATUTORY_INSTRUMENT_PARTIAL_PATH}/{}",
            self.statutory_instrument_id
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

pub struct RequestStatutoryInstrumentBusinessItem {
    pub statutory_instrument_id: String,
}

impl RequestStatutoryInstrumentBusinessItem {
    pub fn new(statutory_instrument_id: impl Into<String>) -> Self {
        Self {
            statutory_instrument_id: statutory_instrument_id.into(),
        }
    }
}

impl ApiRequest for RequestStatutoryInstrumentBusinessItem {
    type Response = Vec<PublishedBusinessItem>;

    fn url(&self) -> impl Into<String> {
        format!(
            "{STATUTORY_INSTRUMENT_API_HOST}/{STATUTORY_INSTRUMENT_PARTIAL_PATH}/{}/BusinessItems",
            self.statutory_instrument_id
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        async {
            Ok(Client::new()
                .get(self.url().into())
                .header("Accept", "application/json")
                .send()
                .await?
                .json()
                .await?)
        }
    }
}

pub struct RequestStatutoryInstrumentList {
    pub name: String,
    pub procedure: String,
    pub house: Option<PublishedHouse>,
    pub skip: Option<i32>,
    pub take: Option<i32>,
}

impl RequestStatutoryInstrumentList {
    pub const fn new(take: i32) -> Self {
        Self {
            name: String::new(),
            procedure: String::new(),
            house: None,
            skip: None,
            take: Some(take),
        }
    }

    pub fn query_string(&self) -> String {
        let mut query_string = String::new();

        if !self.name.is_empty() {
            query_string += &format!("parameters.Name={}&", self.name);
        }

        if !self.procedure.is_empty() {
            query_string += &format!("parameters.Procedure={}&", self.procedure);
        }

        if let Some(house) = self.house.clone() {
            query_string += &format!("parameters.House={house}&");
        }

        if let Some(skip) = self.skip {
            query_string += &format!("parameters.skip={skip}&");
        }

        if let Some(take) = self.take {
            query_string += &format!("parameters.take={take}&");
        }

        query_string.pop();
        query_string
    }
}

impl ApiRequest for RequestStatutoryInstrumentList {
    type Response = Vec<PublishedStatutoryInstrumentSummary>;

    fn url(&self) -> impl Into<String> {
        format!(
            "{STATUTORY_INSTRUMENT_API_HOST}/{STATUTORY_INSTRUMENT_PARTIAL_PATH}?{}",
            self.query_string()
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedStatutoryInstrument {
    #[serde(flatten)]
    pub statutory_instrument: PublishedStatutoryInstrumentSummary,
    pub followed_by_instrument: Option<PublishedStatutoryInstrumentSmall>,
    pub preceded_by_instrument: Option<PublishedStatutoryInstrumentSmall>,
    pub paper_coming_into_force_date: Option<NaiveDateTime>,
    pub paper_coming_into_force_note: Option<String>,
    pub laying_body: PublishedLayingBody,
    pub enabling_acts: Vec<PublishedEnablingAct>,
    pub timelines: Vec<PublishedTimeline>,
    pub link: Option<String>,
    pub business_items: Option<Vec<PublishedBusinessItem>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedStatutoryInstrumentSummary {
    #[serde(flatten)]
    pub statutory_instrument: PublishedStatutoryInstrumentSmall,
    pub paper_prefix: Option<String>,
    pub paper_number: Option<i32>,
    pub paper_year: Option<String>,
    pub paper_made_date: Option<NaiveDateTime>,
    pub commons_laying_date: NaiveDateTime,
    pub lords_laying_date: Option<NaiveDateTime>,
    pub workpackage_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedStatutoryInstrumentSmall {
    pub id: String,
    pub name: String,
    pub procedure: PublishedProcedureDetails,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedProcedure {
    #[serde(flatten)]
    pub procedure: PublishedProcedureDetails,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedProcedureDetails {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedLayingBody {
    pub department_id: Option<i32>,
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedEnablingAct {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedTimeline {
    pub id: String,
    pub date_time: chrono::DateTime<Utc>,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedBusinessItem {
    pub id: String,
    pub statutory_instrument_id: Option<String>,
    pub steps: Vec<String>,
    pub item_date: Option<NaiveDateTime>,
    pub link: Option<String>,
    pub sequence: Option<i32>,
    pub houses: Vec<PublishedHouse>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Display, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PublishedHouse {
    Commons = 1,
    Lords = 2,
}

impl Dataset {
    pub async fn check_statutory_instrument(
        &self,
        si: &mut PublishedStatutoryInstrument,
    ) -> Result<(), PublishedStatutoryInstrumentDataError> {
        if si.statutory_instrument.statutory_instrument.id.len() != 8
            || si
                .statutory_instrument
                .statutory_instrument
                .id
                .chars()
                .any(|c| !c.is_alphanumeric())
        {
            return Err(PublishedStatutoryInstrumentDataError::InvalidID(
                si.statutory_instrument.statutory_instrument.id.clone(),
            ));
        }

        if si
            .statutory_instrument
            .statutory_instrument
            .procedure
            .id
            .len()
            != 8
            || si
                .statutory_instrument
                .statutory_instrument
                .procedure
                .id
                .chars()
                .any(|c| !c.is_alphanumeric())
        {
            return Err(PublishedStatutoryInstrumentDataError::InvalidProcedureID(
                si.statutory_instrument.statutory_instrument.id.clone(),
                si.statutory_instrument
                    .statutory_instrument
                    .procedure
                    .id
                    .clone(),
            ));
        }

        if let Some(followed) = &si.followed_by_instrument {
            if followed.id.len() != 8 || followed.id.chars().any(|c| !c.is_alphanumeric()) {
                return Err(
                    PublishedStatutoryInstrumentDataError::InvalidFollowedStatutoryInstrumentID(
                        si.statutory_instrument.statutory_instrument.id.clone(),
                        followed.id.clone(),
                    ),
                );
            }
        }

        if let Some(preceded) = &si.preceded_by_instrument {
            if preceded.id.len() != 8 || preceded.id.chars().any(|c| !c.is_alphanumeric()) {
                return Err(
                    PublishedStatutoryInstrumentDataError::InvalidPrecededStatutoryInstrumentID(
                        si.statutory_instrument.statutory_instrument.id.clone(),
                        preceded.id.clone(),
                    ),
                );
            }
        }

        if si.laying_body.id.len() != 8 || si.laying_body.id.chars().any(|c| !c.is_alphanumeric()) {
            return Err(PublishedStatutoryInstrumentDataError::InvalidLayingBodyID(
                si.statutory_instrument.statutory_instrument.id.clone(),
                si.laying_body.id.clone(),
            ));
        }

        if let Some(department_id) = &si.laying_body.department_id {
            if *department_id < 1 {
                return Err(
                    PublishedStatutoryInstrumentDataError::InvalidLayingBodyDepartmentID(
                        si.statutory_instrument.statutory_instrument.id.clone(),
                        department_id.clone(),
                    ),
                );
            }
        }

        for act in si.enabling_acts.iter() {
            if act.id.len() != 8 || act.id.chars().any(|c| !c.is_alphanumeric()) {
                return Err(
                    PublishedStatutoryInstrumentDataError::InvalidActOfParliamentID(
                        si.statutory_instrument.statutory_instrument.id.clone(),
                        act.id.clone(),
                    ),
                );
            }
        }

        if si.business_items.is_none() {
            let bis = RequestStatutoryInstrumentBusinessItem::new(
                si.statutory_instrument.statutory_instrument.id.clone(),
            )
            .get()
            .await?;

            for bi in bis.iter() {
                if bi.id.len() != 8 || bi.id.chars().any(|c| !c.is_alphanumeric()) {
                    return Err(
                        PublishedStatutoryInstrumentDataError::InvalidBusinessItemID(
                            si.statutory_instrument.statutory_instrument.id.clone(),
                            bi.id.clone(),
                        ),
                    );
                }
            }

            si.business_items = Some(bis);
        }

        Ok(())
    }

    pub async fn fetch_statutory_instrument(
        &self,
        si_id: impl Into<String>,
    ) -> Result<PublishedStatutoryInstrument, PublishedStatutoryInstrumentDataError> {
        let mut si = RequestStatutoryInstrument::new(si_id.into()).get().await?;
        si.business_items = Some(
            RequestStatutoryInstrumentBusinessItem::new(
                si.statutory_instrument.statutory_instrument.id.clone(),
            )
            .get()
            .await?,
        );

        self.check_statutory_instrument(&mut si).await?;

        tracing::info!(
            "Fetched SI ({}).",
            si.statutory_instrument.statutory_instrument.id.clone()
        );

        Ok(si)
    }

    pub async fn fetch_statutory_instruments(
        &self,
        stored_sis: &Vec<StatutoryInstrument>,
    ) -> Result<Vec<PublishedStatutoryInstrument>, PublishedStatutoryInstrumentDataError> {
        let mut sis = Vec::<PublishedStatutoryInstrument>::new();

        let mut skip = 0;

        loop {
            tracing::info!("SI Collection Round ({})", skip + 1);

            let response = RequestStatutoryInstrumentList {
                name: String::new(),
                procedure: String::new(),
                house: None,
                skip: Some(skip * 75),
                take: Some(75),
            }
            .get()
            .await?;

            if response.is_empty() {
                break;
            }

            if response.iter().all(|si| {
                stored_sis
                    .iter()
                    .filter(|si| {
                        DateTime::now()
                            .saturating_duration_since(si._updated)
                            .as_secs()
                            < 60 * 60 * 24
                    })
                    .any(|s| &s._id == &si.statutory_instrument.id)
                    || sis.iter().any(|s| {
                        si.statutory_instrument.id == s.statutory_instrument.statutory_instrument.id
                    })
            }) {
                break;
            }

            for si in response {
                if bson::DateTime::now()
                    .saturating_duration_since(si.commons_laying_date.and_utc().into())
                    .as_secs()
                    > 60 * 60 * 24 * 120
                {
                    return Ok(sis);
                }

                if stored_sis
                    .iter()
                    .filter(|si| {
                        DateTime::now()
                            .saturating_duration_since(si._updated)
                            .as_secs()
                            < 60 * 60 * 24
                    })
                    .any(|s| &s._id == &si.statutory_instrument.id)
                    || sis.iter().any(|s| {
                        si.statutory_instrument.id == s.statutory_instrument.statutory_instrument.id
                    })
                {
                    tracing::info!(
                        "Skipped Cached SI ({}).",
                        si.statutory_instrument.id.clone()
                    );
                    continue;
                }

                sis.push(
                    self.fetch_statutory_instrument(si.statutory_instrument.id)
                        .await?,
                );
            }

            skip += 1;
        }

        Ok(sis)
    }
}
