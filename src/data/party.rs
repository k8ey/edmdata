use serde::{Deserialize, Serialize};

use crate::api::member::PublishedParty;

pub type PartyID = u32;

pub const PARTY_COLLECTION: &str = "parties";

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PartyExtra {}

impl From<&PublishedParty> for PartyExtra {
    fn from(_value: &PublishedParty) -> Self {
        Self {}
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Party {
    pub _id: PartyID,
    pub name: String,
    pub abbreviation: String,
    pub background_colour: String,
    pub foreground_colour: String,
    pub is_independent_party: bool,
    pub government_type: Option<u8>,
}

impl From<PublishedParty> for Party {
    fn from(value: PublishedParty) -> Self {
        Self {
            _id: value.id as PartyID,
            name: value.name,
            abbreviation: value.abbreviation.unwrap_or(String::new()),
            background_colour: value.background_colour.unwrap_or(String::new()),
            foreground_colour: value.foreground_colour.unwrap_or(String::new()),
            is_independent_party: value.is_independent_party,
            government_type: value.government_type.map(|r#type| r#type as u8),
        }
    }
}
