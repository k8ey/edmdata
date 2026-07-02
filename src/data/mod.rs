use std::{
    ops::Sub,
    time::{Duration, SystemTime},
};

use bson::{DateTime, Document};
use derive_more::Display;
use mongodb::{Client, Collection, Database, bson::doc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::data::{
    edm::{EDM_COLLECTION, EarlyDayMotion, EarlyDayMotionExtra},
    member::{MEMBER_COLLECTION, Member, MemberExtra},
    party::{PARTY_COLLECTION, Party, PartyExtra},
    si::{SI_COLLECTION, StatutoryInstrument, StatutoryInstrumentExtra},
};

pub mod edm;
pub mod member;
pub mod party;
pub mod si;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatasetItem<T, E> {
    pub item: T,
    pub extra: E,
}

pub struct Dataset {
    pub client: Client,
    pub database: Database,

    pub edms: Collection<DatasetItem<EarlyDayMotion, EarlyDayMotionExtra>>,
    pub members: Collection<DatasetItem<Member, MemberExtra>>,
    pub parties: Collection<DatasetItem<Party, PartyExtra>>,
    pub sis: Collection<DatasetItem<StatutoryInstrument, StatutoryInstrumentExtra>>,
}

impl Dataset {
    pub async fn new() -> Self {
        let uri = std::env::var("MONGODB_URI").unwrap_or("mongodb://127.0.0.1".to_owned());
        let client = Client::with_uri_str(uri).await.unwrap();
        let database = client.database("datasets");
        let (edms, members, parties, sis) = (
            database.collection(EDM_COLLECTION),
            database.collection(MEMBER_COLLECTION),
            database.collection(PARTY_COLLECTION),
            database.collection(SI_COLLECTION),
        );

        tracing::info!("Connected to the Dataset!");

        Self {
            client,
            database,
            edms,
            members,
            parties,
            sis,
        }
    }

    pub fn cached(mut doc: Document, hours: u64) -> Document {
        doc.extend(doc! {
            "item._updated": {
                "$gt": DateTime::from_system_time(SystemTime::now().sub(Duration::from_hours(hours))),
            },
        });

        doc
    }
}

#[derive(
    Clone, Debug, Deserialize_repr, Serialize_repr, Display, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u8)]
pub enum House {
    Commons = 1,
    Lords = 2,

    None = 255,
}

impl From<u8> for House {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Commons,
            2 => Self::Lords,
            x => panic!("Unknown House variant: ({x})"),
        }
    }
}
