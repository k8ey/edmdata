use bson::DateTime;
use chrono::NaiveDateTime;
use derive_more::{Display, Error, From};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiError, ApiRequest, member::RequestMember},
    data::{Dataset, edm::EarlyDayMotion},
};

pub const EARLY_DAY_MOTION_API_HOST: &str = "https://oralquestionsandmotions-api.parliament.uk";

#[derive(Debug, Display, Error, From)]
pub enum PublishedEarlyDayMotionDataError {
    #[display("EDM ID was < 1: ({_0})")]
    #[from(skip)]
    InvalidID(#[error(not(source))] i32),

    #[display("EDM ({_0}) Signature Member ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidSignatureID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("EDM ({_0}) Member ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidMemberID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("EDM ({_0}) Primary Sponsor ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidPrimarySponsorID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("EDM ({_0}) Statutory Instrument ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidStatutoryInstrumentID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("EDM ({_0}) Amendment EDM ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidAmendedMotionID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("EDM ({_0}) Member ({_1}) Party ID was < 1: ({_2})")]
    #[from(skip)]
    InvalidPartyID(
        #[error(not(source))] i32,
        #[error(not(source))] i32,
        #[error(not(source))] i32,
    ),

    #[display("Amendment EDM is missing Amended EDM ID for Amendment EDM: ({_0})")]
    #[from(skip)]
    MissingAmendedMotionID(#[error(not(source))] i32),

    #[display(
        "EDM ({_0}) Member IDs for Primary Sponsor do not match. Member ID: ({_1}), Sponsor Member ID: ({_2})"
    )]
    #[from(skip)]
    MismatchedMemberID(
        #[error(not(source))] i32,
        #[error(not(source))] i32,
        #[error(not(source))] i32,
    ),

    #[display(
        "EDM ({_0}) Signature withdrawal does not match. IsWithdrawn: ({_1}), WithdrawnDate: ({_2:?})"
    )]
    #[from(skip)]
    WithdrawnSignatureMismatch(
        #[error(not(source))] i32,
        #[error(not(source))] bool,
        #[error(not(source))] Option<NaiveDateTime>,
    ),

    #[display("Amended EDM's ({_0}) amendment order does not match date order. EDM ID: ({_1})")]
    #[from(skip)]
    AmendmentOrderMismatch(#[error(not(source))] i32, #[error(not(source))] i32),

    MongoError(mongodb::error::Error),

    ApiError(ApiError),
}

pub struct RequestEarlyDayMotion {
    pub early_day_motion_id: i32,
}

impl RequestEarlyDayMotion {
    pub const fn new(early_day_motion_id: i32) -> Self {
        Self {
            early_day_motion_id,
        }
    }
}

impl ApiRequest for RequestEarlyDayMotion {
    type Response = PublishedEarlyDayMotionDetails;

    fn url(&self) -> impl Into<String> {
        format!(
            "{EARLY_DAY_MOTION_API_HOST}/EarlyDayMotion/{}",
            self.early_day_motion_id
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

#[derive(Clone, Debug, Display, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublishedEarlyDayMotionStatus {
    Published,
    Withdrawn,
}

#[derive(Clone, Debug, Display, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderBy {
    DateTabledAsc,
    DateTabledDesc,
    TitleAsc,
    TitleDesc,
    SignatureCountAsc,
    SignatureCountDesc,
}

pub struct RequestEarlyDayMotionList {
    pub early_day_motion_ids: Vec<i32>,
    pub tabled_start_date: Option<NaiveDateTime>,
    pub tabled_end_date: Option<NaiveDateTime>,
    pub statuses: Vec<PublishedEarlyDayMotionStatus>,
    pub order_by: Option<OrderBy>,
    pub skip: Option<i32>,
    pub take: Option<i32>,
}

impl RequestEarlyDayMotionList {
    pub fn new(early_day_motion_ids: Vec<i32>) -> Self {
        Self {
            early_day_motion_ids,
            tabled_start_date: None,
            tabled_end_date: None,
            statuses: Vec::new(),
            order_by: None,
            skip: None,
            take: None,
        }
    }

    pub fn query_string(&self) -> String {
        let mut query_string = String::new();

        if !self.early_day_motion_ids.is_empty() {
            let ids = self
                .early_day_motion_ids
                .iter()
                .map(|edm_id| format!("parameters.edmIds={edm_id}&"))
                .collect::<String>();
            query_string += &ids;
        }

        if let Some(tabled_start_date) = self.tabled_start_date {
            query_string += &format!("parameters.tabledStartDate={}&", tabled_start_date);
        }

        if let Some(tabled_end_date) = self.tabled_end_date {
            query_string += &format!("parameters.tabledEndDate={}&", tabled_end_date);
        }

        if !self.statuses.is_empty() {
            let statuses = self
                .statuses
                .iter()
                .map(|status| format!("parameters.statuses={status}&"))
                .collect::<String>();
            query_string += &statuses;
        }

        if let Some(order_by) = &self.order_by {
            query_string += &format!("parameters.orderBy={}&", order_by);
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

impl ApiRequest for RequestEarlyDayMotionList {
    type Response = Vec<PublishedEarlyDayMotion>;

    fn url(&self) -> impl Into<String> {
        format!(
            "{EARLY_DAY_MOTION_API_HOST}/EarlyDayMotions/list?{}",
            self.query_string()
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub struct PublishedEarlyDayMotionDetails {
    pub sponsors: Vec<PublishedEarlyDayMotionSponsor>,
    pub amendments: Vec<PublishedEarlyDayMotionDetails>,
    #[serde(flatten)]
    pub motion: PublishedEarlyDayMotion,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub struct PublishedEarlyDayMotion {
    pub id: i32,
    pub status: i32,
    pub status_date: NaiveDateTime,
    pub member_id: i32,
    pub primary_sponsor: MemberForDate,
    pub title: Option<String>,
    pub motion_text: Option<String>,
    pub amendment_to_motion_id: Option<i32>,
    #[serde(rename = "UIN")]
    pub uin: i32,
    pub amendment_suffix: Option<String>,
    pub date_tabled: NaiveDateTime,
    pub praying_against_negative_statutory_instrument_id: Option<i32>,
    pub statutory_instrument_number: Option<i32>,
    pub statutory_instrument_year: Option<String>,
    pub statutory_instrument_title: Option<String>,
    #[serde(rename = "UINWithAmendmentSuffix")]
    pub uin_with_amendment_suffix: String,
    pub sponsors_count: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub struct PublishedEarlyDayMotionSponsor {
    pub id: i32,
    pub member_id: i32,
    pub member: MemberForDate,
    pub sponsoring_order: Option<i32>,
    pub created_when: NaiveDateTime,
    pub is_withdrawn: bool,
    pub withdrawn_date: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub struct MemberForDate {
    pub mnis_id: i32,
    pub pims_id: Option<i32>,
    pub name: String,
    pub list_as: String,
    pub constituency: Option<String>,
    pub status: String,
    pub party: Option<String>,
    pub party_id: Option<i32>,
    pub party_colour: Option<String>,
    pub photo_url: Option<String>,
}

impl Dataset {
    pub async fn check_early_day_motion(
        &self,
        edm: &mut PublishedEarlyDayMotionDetails,
    ) -> Result<(), PublishedEarlyDayMotionDataError> {
        if edm.motion.id < 1 {
            return Err(PublishedEarlyDayMotionDataError::InvalidID(edm.motion.id));
        }

        if edm.motion.member_id < 1 {
            return Err(PublishedEarlyDayMotionDataError::InvalidMemberID(
                edm.motion.id,
                edm.motion.member_id,
            ));
        }

        if edm.motion.primary_sponsor.mnis_id < 1 {
            return Err(PublishedEarlyDayMotionDataError::InvalidPrimarySponsorID(
                edm.motion.id,
                edm.motion.primary_sponsor.mnis_id,
            ));
        }

        if edm.motion.member_id != edm.motion.primary_sponsor.mnis_id {
            return Err(PublishedEarlyDayMotionDataError::MismatchedMemberID(
                edm.motion.id,
                edm.motion.member_id,
                edm.motion.primary_sponsor.mnis_id,
            ));
        }

        for sponsor in &edm.sponsors {
            if sponsor.member_id < 1 {
                return Err(PublishedEarlyDayMotionDataError::InvalidSignatureID(
                    edm.motion.id,
                    sponsor.member_id,
                ));
            }

            if sponsor.is_withdrawn != sponsor.withdrawn_date.is_some() {
                return Err(
                    PublishedEarlyDayMotionDataError::WithdrawnSignatureMismatch(
                        edm.motion.id,
                        sponsor.is_withdrawn,
                        sponsor.withdrawn_date,
                    ),
                );
            }
        }

        if let Some(statutory_instrument_id) =
            edm.motion.praying_against_negative_statutory_instrument_id
            && statutory_instrument_id < 1
        {
            return Err(
                PublishedEarlyDayMotionDataError::InvalidStatutoryInstrumentID(
                    edm.motion.id,
                    statutory_instrument_id,
                ),
            );
        }

        for member in &mut edm.sponsors {
            match member.member.party_id {
                Some(party_id) => {
                    if party_id < 1 {
                        return Err(PublishedEarlyDayMotionDataError::InvalidPartyID(
                            edm.motion.id,
                            member.member_id,
                            party_id,
                        ));
                    }
                }
                None => {
                    member.member.party_id = RequestMember::new(member.member_id)
                        .get()
                        .await?
                        .latest_party
                        .map(|party| party.id);
                }
            };
        }

        for amendment in &edm.amendments {
            match amendment.motion.amendment_to_motion_id {
                Some(amendment_to_motion_id) => {
                    if amendment_to_motion_id < 1 {
                        return Err(PublishedEarlyDayMotionDataError::InvalidAmendedMotionID(
                            edm.motion.id,
                            amendment_to_motion_id,
                        ));
                    }
                }
                None => {
                    return Err(PublishedEarlyDayMotionDataError::MissingAmendedMotionID(
                        amendment.motion.id,
                    ));
                }
            };
        }

        Ok(())
    }

    pub async fn fetch_early_day_motion(
        &self,
        edm_id: i32,
    ) -> Result<PublishedEarlyDayMotionDetails, PublishedEarlyDayMotionDataError> {
        let mut edm = RequestEarlyDayMotion::new(edm_id).get().await?;
        self.check_early_day_motion(&mut edm).await?;

        tracing::info!("Fetched EDM ({edm_id}).");
        Ok(edm)
    }

    pub async fn fetch_early_day_motions(
        &self,
        stored_edms: &Vec<EarlyDayMotion>,
    ) -> Result<Vec<PublishedEarlyDayMotionDetails>, PublishedEarlyDayMotionDataError> {
        let mut edms: Vec<PublishedEarlyDayMotionDetails> = Vec::new();

        let max_id = RequestEarlyDayMotionList {
            early_day_motion_ids: Vec::new(),
            tabled_start_date: None,
            tabled_end_date: None,
            statuses: vec![
                PublishedEarlyDayMotionStatus::Published,
                PublishedEarlyDayMotionStatus::Withdrawn,
            ],
            order_by: Some(OrderBy::DateTabledDesc),
            skip: None,
            take: Some(1),
        }
        .get()
        .await?
        .into_iter()
        .next()
        .map(|edm| edm.id);

        let max_id = match max_id {
            Some(max_id) => max_id,
            None => panic!(),
        };

        for id in 1..=max_id {
            if stored_edms
                .iter()
                .filter(|edm| {
                    DateTime::now()
                        .saturating_duration_since(edm._updated)
                        .as_secs()
                        < 60 * 60 * 24
                })
                .any(|edm| edm._id == id as u32)
                || edms.iter().any(|e| id == e.motion.id)
            {
                tracing::info!("Skipped Cached EDM ({}).", id);
                continue;
            }

            edms.push(self.fetch_early_day_motion(id).await?);
        }

        Ok(edms)
    }
}
