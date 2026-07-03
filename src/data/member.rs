use bson::DateTime;
use serde::{Deserialize, Serialize};

use crate::{
    api::member::{
        PublishedMember, PublishedMemberBiography, PublishedMemberBiographyItem,
        PublishedMemberRepresentation,
    },
    data::{House, edm::EarlyDayMotionID, party::PartyID},
};

pub const MEMBER_COLLECTION: &str = "members";

pub type MemberID = u32;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberExtra {}

impl From<&PublishedMember> for MemberExtra {
    fn from(_value: &PublishedMember) -> Self {
        Self {}
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
    pub _id: MemberID,
    pub name: String,
    pub display_name: String,
    pub full_title: String,
    pub addressed_as: String,
    pub party_id: PartyID,
    pub gender: String,

    pub biography: MemberBiography,
    pub edms: Vec<EarlyDayMotionID>,

    pub _updated: DateTime,
}

impl From<PublishedMember> for Member {
    fn from(value: PublishedMember) -> Self {
        let biography = value
            .biography
            .map(|biography| biography.into())
            .unwrap_or(MemberBiography::default());

        Self {
            _id: value.id as MemberID,
            name: value.name_list_as.unwrap_or(String::new()),
            display_name: value.name_display_as.unwrap_or(String::new()),
            full_title: value.name_full_title.unwrap_or(String::new()),
            addressed_as: value.name_address_as.unwrap_or(String::new()),
            party_id: value
                .latest_party
                .map(|party| party.id as PartyID)
                .unwrap_or(1050),
            gender: value.gender.unwrap_or(String::new()),
            biography,
            edms: value
                .edms
                .unwrap_or(Vec::new())
                .into_iter()
                .map(|edm| edm.id as EarlyDayMotionID)
                .collect(),
            _updated: DateTime::now(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberBiography {
    pub representations: Vec<MemberRepresentation>,
    pub elections_contested: Vec<MemberBiographyItem>,
    pub house_memberships: Vec<MemberBiographyItem>,
    pub government_posts: Vec<MemberBiographyItem>,
    pub opposition_posts: Vec<MemberBiographyItem>,
    pub other_posts: Vec<MemberBiographyItem>,
    pub party_affiliations: Vec<MemberBiographyItem>,
    pub committee_memberships: Vec<MemberBiographyItem>,
}

impl From<PublishedMemberBiography> for MemberBiography {
    fn from(value: PublishedMemberBiography) -> Self {
        Self {
            representations: value
                .representations
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            elections_contested: value
                .elections_contested
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            house_memberships: value
                .house_memberships
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            government_posts: value
                .government_posts
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            opposition_posts: value
                .opposition_posts
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            other_posts: value
                .other_posts
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            party_affiliations: value
                .party_affiliations
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
            committee_memberships: value
                .committee_memberships
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberBiographyItem {
    pub house: House,
    pub name: String,
    // TODO: Rename to _id?
    pub id: u32,
    pub start_date: DateTime,
    pub end_date: Option<DateTime>,
    pub additional_info: String,
    pub additional_info_link: String,
}

impl From<PublishedMemberBiographyItem> for MemberBiographyItem {
    fn from(value: PublishedMemberBiographyItem) -> Self {
        Self {
            house: value
                .house
                .map(|house| (house as u8).into())
                .unwrap_or(House::None),
            name: value.name,
            id: value.id as u32,
            start_date: value.start_date.and_utc().into(),
            end_date: value.end_date.map(|dt| dt.and_utc().into()),
            additional_info: value.additional_info.unwrap_or(String::new()),
            additional_info_link: value.additional_info_link.unwrap_or(String::new()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberRepresentation {
    pub constituency_start: DateTime,
    pub constituency_end: Option<DateTime>,
    #[serde(flatten)]
    pub item: MemberBiographyItem,
}

impl From<PublishedMemberRepresentation> for MemberRepresentation {
    fn from(value: PublishedMemberRepresentation) -> Self {
        Self {
            constituency_start: value.constituency_start.and_utc().into(),
            constituency_end: value.constituency_end.map(|dt| dt.and_utc().into()),
            item: value.item.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HouseMembership {
    pub house: House,
    pub description: String,
    pub constituency: String,
    pub constituency_id: u32,

    pub start_date: DateTime,
    pub end_date: Option<DateTime>,
    pub end_reason: String,
    pub end_reason_id: u32,
    pub end_note: String,
}
