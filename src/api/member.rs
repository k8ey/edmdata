use bson::DateTime;
use chrono::NaiveDateTime;
use derive_more::{Display, Error, From};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    api::{ApiError, ApiRequest},
    data::{Dataset, member::Member},
};

pub const MEMBER_API_HOST: &str = "https://members-api.parliament.uk";
pub const MEMBER_API_PARTIAL_PATH: &str = "api";

#[derive(Debug, Display, Error, From)]
pub enum PublishedMemberDataError {
    #[display("Mnis ID (Member ID) was < 1: ({_0})")]
    #[from(skip)]
    InvalidID(#[error(not(source))] i32),

    #[display("Member ({_0}) Pims ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidPimsID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("Member ({_0}) EDM ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidEarlyDayMotionID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("Member ({_0}) Membership Constituency ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidMembershipConstituencyID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("Member ({_0}) Membership end reason ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidMembershipEndReasonID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("Member ({_0}) Membership BiographyItem ID was < 1: ({_1})")]
    #[from(skip)]
    InvalidBiographyItemID(#[error(not(source))] i32, #[error(not(source))] i32),

    #[display("Member ({_0}) start date ({_1}) was < end date ({_2})")]
    #[from(skip)]
    InvalidDates(
        #[error(not(source))] i32,
        #[error(not(source))] NaiveDateTime,
        #[error(not(source))] NaiveDateTime,
    ),

    MongoError(mongodb::error::Error),

    ApiError(ApiError),
}

pub struct RequestMember {
    pub member_id: i32,
}

impl RequestMember {
    pub const fn new(member_id: i32) -> Self {
        Self { member_id }
    }
}

impl ApiRequest for RequestMember {
    type Response = PublishedMember;

    fn url(&self) -> impl Into<String> {
        format!(
            "{MEMBER_API_HOST}/{MEMBER_API_PARTIAL_PATH}/Members/{}",
            self.member_id
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

pub struct RequestMemberEdmList {
    pub member_id: i32,
    pub page: i32,
}

impl RequestMemberEdmList {
    pub const fn new(member_id: i32, page: i32) -> Self {
        Self { member_id, page }
    }
}

impl ApiRequest for RequestMemberEdmList {
    type Response = Vec<PublishedMemberEdm>;

    fn url(&self) -> impl Into<String> {
        format!(
            "{MEMBER_API_HOST}/{MEMBER_API_PARTIAL_PATH}/Members/{}/Edms?page={}",
            self.member_id, self.page,
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

pub struct RequestMemberBiography {
    pub member_id: i32,
}

impl RequestMemberBiography {
    pub const fn new(member_id: i32) -> Self {
        Self { member_id }
    }
}

impl ApiRequest for RequestMemberBiography {
    type Response = PublishedMemberBiography;

    fn url(&self) -> impl Into<String> {
        format!(
            "{MEMBER_API_HOST}/{MEMBER_API_PARTIAL_PATH}/Members/{}/Biography",
            self.member_id
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

pub struct RequestMemberList {
    pub name: Option<String>,
    pub house: Option<PublishedHouse>,
    pub is_current_member: Option<bool>,
    pub skip: Option<i32>,
    pub take: Option<i32>,
}

impl RequestMemberList {
    pub const fn new(skip: i32, take: i32) -> Self {
        Self {
            name: None,
            house: None,
            is_current_member: None,
            skip: Some(skip),
            take: Some(take),
        }
    }

    pub fn query_string(&self) -> String {
        let mut query_string = String::new();

        if let Some(name) = self.name.clone() {
            query_string += &format!("Name={name}&");
        }

        if let Some(house) = self.house.clone() {
            query_string += &format!("House={}&", house as u8);
        }

        if let Some(is_current_member) = self.is_current_member.clone() {
            query_string += &format!("IsCurrentMember={}&", is_current_member);
        }

        if let Some(skip) = self.skip {
            query_string += &format!("skip={skip}&");
        }

        if let Some(take) = self.take {
            query_string += &format!("take={take}&");
        }

        query_string.pop();
        query_string
    }
}

impl ApiRequest for RequestMemberList {
    type Response = Vec<PublishedMember>;

    fn url(&self) -> impl Into<String> {
        format!(
            "{MEMBER_API_HOST}/{MEMBER_API_PARTIAL_PATH}/Members/Search?{}",
            self.query_string()
        )
    }

    fn get(&self) -> impl Future<Output = Result<Self::Response, ApiError>> {
        self.get_response()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedMemberEdm {
    pub title: Option<String>,
    pub number: Option<String>,
    pub is_prayer: Option<bool>,
    pub is_amendment: Option<bool>,
    pub id: i32,
    pub date_tabled: NaiveDateTime,
    pub sponsors_count: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedMember {
    pub id: i32,
    pub name_list_as: Option<String>,
    pub name_display_as: Option<String>,
    pub name_full_title: Option<String>,
    pub name_address_as: Option<String>,
    pub latest_party: Option<PublishedParty>,
    pub gender: Option<String>,
    pub latest_house_membership: PublishedHouseMembership,
    pub thumbnail_url: Option<String>,

    pub biography: Option<PublishedMemberBiography>,
    pub edms: Option<Vec<PublishedMemberEdm>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedParty {
    pub id: i32,
    pub name: String,
    pub abbreviation: Option<String>,
    pub background_colour: Option<String>,
    pub foreground_colour: Option<String>,
    pub is_lords_main_party: Option<bool>,
    pub is_lords_spiritual_party: Option<bool>,
    pub government_type: Option<i32>,
    pub is_independent_party: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedHouseMembership {
    pub membership_from: Option<String>,
    pub membership_from_id: Option<i32>,
    pub house: Option<PublishedHouse>,
    pub membership_start_date: NaiveDateTime,
    pub membership_end_date: Option<NaiveDateTime>,
    pub membership_end_reason: Option<String>,
    pub membership_end_reason_notes: Option<String>,
    pub membership_end_reason_id: Option<i32>,
    pub membership_status: Option<PublishedHouseMembershipStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedHouseMembershipStatus {
    pub status_is_active: bool,
    pub status_description: String,
    pub status_notes: Option<String>,
    pub status_id: Option<i32>,
    pub status: Option<i32>,
    pub status_start_date: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedMemberBiography {
    pub representations: Vec<PublishedMemberRepresentation>,
    pub elections_contested: Vec<PublishedMemberBiographyItem>,
    pub house_memberships: Vec<PublishedMemberBiographyItem>,
    pub government_posts: Vec<PublishedMemberBiographyItem>,
    pub opposition_posts: Vec<PublishedMemberBiographyItem>,
    pub other_posts: Vec<PublishedMemberBiographyItem>,
    pub party_affiliations: Vec<PublishedMemberBiographyItem>,
    pub committee_memberships: Vec<PublishedMemberBiographyItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedMemberBiographyItem {
    pub house: Option<PublishedHouse>,
    pub name: String,
    pub id: i32,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub additional_info: Option<String>,
    pub additional_info_link: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PublishedMemberRepresentation {
    pub constituency_start: NaiveDateTime,
    pub constituency_end: Option<NaiveDateTime>,
    #[serde(flatten)]
    pub item: PublishedMemberBiographyItem,
}

#[derive(
    Clone, Debug, Deserialize_repr, Serialize_repr, Display, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(i32)]
pub enum PublishedHouse {
    Commons = 1,
    Lords = 2,
}

impl Dataset {
    pub async fn check_member(
        &self,
        member: &mut PublishedMember,
    ) -> Result<(), PublishedMemberDataError> {
        if member.id < 1 {
            return Err(PublishedMemberDataError::InvalidID(member.id));
        }

        if let Some(edms) = &member.edms {
            for edm in edms {
                if edm.id < 1 {
                    return Err(PublishedMemberDataError::InvalidEarlyDayMotionID(
                        member.id, edm.id,
                    ));
                }
            }
        }

        if let Some(membership_from_id) = member.latest_house_membership.membership_from_id
            && membership_from_id < 1
        {
            return Err(PublishedMemberDataError::InvalidMembershipConstituencyID(
                member.id,
                membership_from_id,
            ));
        }

        if let Some(membership_end_reason_id) =
            member.latest_house_membership.membership_end_reason_id
            && membership_end_reason_id < 1
        {
            return Err(PublishedMemberDataError::InvalidMembershipEndReasonID(
                member.id,
                membership_end_reason_id,
            ));
        }

        if let Some(biography) = &member.biography {
            for item in &biography.committee_memberships {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.elections_contested {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.government_posts {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.house_memberships {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.opposition_posts {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.other_posts {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.party_affiliations {
                if item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id, item.id,
                    ));
                }
            }

            for item in &biography.representations {
                if item.item.id < 1 {
                    return Err(PublishedMemberDataError::InvalidBiographyItemID(
                        member.id,
                        item.item.id,
                    ));
                }
            }
        }

        Ok(())
    }

    pub async fn fetch_member_edms(
        &self,
        member_id: i32,
        cached: &Vec<Member>,
    ) -> Result<Vec<PublishedMemberEdm>, PublishedMemberDataError> {
        let mut edms = Vec::<PublishedMemberEdm>::new();

        let mut page = 1;

        loop {
            let response = match RequestMemberEdmList::new(member_id, page).get().await {
                Ok(response) => response,
                Err(_) => break,
            };

            if response.is_empty() {
                break;
            }

            if response.iter().all(|edm| {
                cached
                    .iter()
                    .find(|m| m.edms.contains(&(edm.id as u32)))
                    .is_some()
                    || edms.iter().find(|e| edm.id == e.id).is_some()
            }) {
                break;
            }

            for edm in response {
                if DateTime::now()
                    .saturating_duration_since(edm.date_tabled.and_utc().into())
                    .as_secs()
                    > 60 * 60 * 24 * 365
                {
                    return Ok(edms);
                }

                edms.push(edm.into());
            }

            page += 1;
        }

        Ok(edms)
    }

    pub async fn fetch_member_biography(
        &self,
        member_id: i32,
    ) -> Result<PublishedMemberBiography, PublishedMemberDataError> {
        Ok(RequestMemberBiography::new(member_id).get().await?)
    }

    pub async fn fetch_member(
        &self,
        member_id: i32,
        cached: &Vec<Member>,
    ) -> Result<PublishedMember, PublishedMemberDataError> {
        let mut member = RequestMember::new(member_id).get().await?;
        member.biography = Some(self.fetch_member_biography(member_id).await?);
        member.edms = Some(self.fetch_member_edms(member_id, cached).await?);

        self.check_member(&mut member).await?;

        Ok(member)
    }

    pub async fn fetch_members(
        &self,
        cached: &Vec<Member>,
    ) -> Result<Vec<PublishedMember>, PublishedMemberDataError> {
        let mut members = Vec::<PublishedMember>::new();

        let mut skip = 0;

        loop {
            tracing::info!("Member Collection Round ({})", skip + 1);

            let response = RequestMemberList {
                name: None,
                house: None,
                is_current_member: Some(true),
                skip: Some(skip * 20),
                take: Some(20),
            }
            .get()
            .await?;
            if response.is_empty() {
                break;
            }

            for mut member in response {
                if cached.iter().find(|m| m._id == member.id as u32).is_some()
                    || members.iter().find(|m| member.id == m.id).is_some()
                {
                    tracing::info!("Skipped Cached Member ({}).", member.id);
                    continue;
                }

                member.biography = Some(self.fetch_member_biography(member.id).await?);
                member.edms = Some(self.fetch_member_edms(member.id, cached).await?);
                self.check_member(&mut member).await?;

                tracing::info!(
                    "Fetched Member ({}): ({}).",
                    member.id,
                    member.name_display_as.clone().unwrap_or("".to_owned())
                );

                members.push(member.into());
            }

            skip += 1;
        }

        Ok(members)
    }
}
