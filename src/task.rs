use derive_more::{Display, Error, From};
use futures_util::TryStreamExt;
use itertools::Itertools;
use mongodb::bson::doc;

use crate::{
    api::{
        edm::{PublishedEarlyDayMotionDataError, PublishedEarlyDayMotionDetails},
        member::PublishedMemberDataError,
        si::PublishedStatutoryInstrumentDataError,
    },
    data::{
        Dataset,
        edm::{EarlyDayMotion, EarlyDayMotionExtra, SourceKind},
        member::{Member, MemberExtra},
        party::{Party, PartyExtra},
        si::{StatutoryInstrument, StatutoryInstrumentExtra},
    },
};

#[derive(Debug, Display, Error, From)]
pub enum TaskError {
    #[display(
        "Unable to find ({_1}) the related SI for EDM ({_0}). PrayingAgainstNegativeStatutoryInstrumentId: ({_2:?}), StatutoryInstrumentNumber ({_3:?}), StatutoryInstrumentTitle ({_4:?}), StatutoryInstrumentYear ({_5:?})"
    )]
    #[from(skip)]
    StatutoryInstrumentUnknown(
        #[error(not(source))] i32,
        #[error(not(source))] usize,
        #[error(not(source))] Option<i32>,
        #[error(not(source))] Option<i32>,
        #[error(not(source))] Option<String>,
        #[error(not(source))] Option<String>,
    ),

    PublishedEarlyDayMotionDataError(PublishedEarlyDayMotionDataError),

    PublishedMemberDataError(PublishedMemberDataError),

    PublishedStatutoryInstrumentDataError(PublishedStatutoryInstrumentDataError),

    JsonError(serde_json::Error),

    MongoError(mongodb::error::Error),

    MongoBsonError(mongodb::bson::error::Error),
}

impl Dataset {
    pub async fn run_tasks(&self) -> Result<(), TaskError> {
        self.run_refresh_tasks().await?;
        self.run_normalize_tasks().await?;

        Ok(())
    }

    pub async fn run_refresh_tasks(&self) -> Result<(), TaskError> {
        let cached = self
            .sis
            .find(Self::cached(doc! {}, 24 * 7))
            .await?
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .map(|si| si.item)
            .collect::<Vec<_>>();
        let sis = self.fetch_statutory_instruments(&cached).await?;

        let values = sis
            .into_iter()
            .map(|si| {
                (
                    StatutoryInstrumentExtra::from(&si),
                    StatutoryInstrument::from(si),
                )
            })
            .collect::<Vec<_>>();

        for value in values.iter().map(|(_, si)| si) {
            self.sis
                .update_one(
                    doc! {
                        "item._id": value._id.clone(),
                    },
                    doc! {
                        "$set": {
                            "item": bson::serialize_to_document(value)?,
                        },
                    },
                )
                .upsert(true)
                .await?;
        }
        for (value, si) in values.iter().map(|(extra, si)| (extra, si)) {
            self.sis
                .update_one(
                    doc! {
                        "item._id": si._id.clone(),
                        "extra": null,
                    },
                    doc! {
                        "$set": {
                            "extra": bson::serialize_to_document(value)?,
                        },
                    },
                )
                .await?;
        }

        let cached = self
            .members
            .find(Self::cached(doc! {}, 24))
            .await?
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .map(|member| member.item)
            .collect::<Vec<_>>();
        let members = self.fetch_members(&cached).await?;

        let values = members
            .into_iter()
            .unique_by(|member| member.id)
            .map(|member| {
                (
                    member
                        .latest_party
                        .clone()
                        .map(|party| PartyExtra::from(&party)),
                    member.latest_party.clone().map(|party| Party::from(party)),
                    MemberExtra::from(&member),
                    Member::from(member),
                )
            })
            .collect::<Vec<_>>();

        for value in values.iter().map(|(_, party, _, _)| party) {
            let value = match value {
                Some(value) => value,
                None => continue,
            };

            self.parties
                .update_one(
                    doc! {
                        "item._id": value._id,
                    },
                    doc! {
                        "$set": {
                            "item": bson::serialize_to_document(value)?,
                        },
                    },
                )
                .upsert(true)
                .await?;
        }
        for (value, party) in values.iter().map(|(extra, party, _, _)| (extra, party)) {
            let (value, party) = match (value, party) {
                (Some(value), Some(party)) => (value, party),
                _ => continue,
            };

            self.parties
                .update_one(
                    doc! {
                        "item._id": party._id,
                        "extra": null,
                    },
                    doc! {
                        "$set": {
                            "extra": bson::serialize_to_document(value)?,
                        },
                    },
                )
                .await?;
        }

        for value in values.iter().map(|(_, _, _, member)| member) {
            self.members
                .update_one(
                    doc! {
                        "item._id": value._id,
                    },
                    doc! {
                        "$set": {
                            "item": bson::serialize_to_document(value)?,
                        },
                    },
                )
                .upsert(true)
                .await?;
        }
        for (value, member) in values.iter().map(|(_, _, extra, member)| (extra, member)) {
            self.members
                .update_one(
                    doc! {
                        "item._id": member._id,
                        "extra": null,
                    },
                    doc! {
                        "$set": {
                            "extra": bson::serialize_to_document(value)?,
                        },
                    },
                )
                .await?;
        }

        for edm_id in self
            .members
            .find(Self::cached(doc! {}, 24))
            .await?
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .flat_map(|member| member.item.edms)
            .unique()
        {
            let published_edm = self.fetch_early_day_motion(edm_id as i32).await?;

            let si_id = self.normalize_opposing_edm(&published_edm).await?;
            let edm = published_edm.into();

            let mut edm_extra = EarlyDayMotionExtra::from(&edm);
            edm_extra.opposes_statutory_instrument = si_id;
            let edm = EarlyDayMotion::from(edm);

            self.edms
                .update_one(
                    doc! {
                        "item._id": edm._id,
                    },
                    doc! {
                        "$set": {
                            "item": bson::serialize_to_document(&edm)?,
                        },
                    },
                )
                .upsert(true)
                .await?;

            self.edms
                .update_one(
                    doc! {
                        "item._id": edm._id,
                        "extra": null,
                    },
                    doc! {
                        "$set": {
                            "extra": bson::serialize_to_document(&edm_extra)?,
                        },
                    },
                )
                .await?;
        }

        Ok(())
    }

    pub async fn run_normalize_tasks(&self) -> Result<(), TaskError> {
        self.normalize_si_name_comparison().await?;

        Ok(())
    }

    pub async fn normalize_opposing_edm(
        &self,
        edm: &PublishedEarlyDayMotionDetails,
    ) -> Result<Option<String>, TaskError> {
        let is_prayer = edm
            .motion
            .praying_against_negative_statutory_instrument_id
            .is_some()
            || edm.motion.statutory_instrument_number.is_some()
            || edm.motion.statutory_instrument_title.is_some()
            || edm.motion.statutory_instrument_year.is_some();

        let is_within_dataset = match (
            edm.motion
                .statutory_instrument_year
                .as_ref()
                .map(|year| year.parse::<i32>().ok())
                .flatten(),
            edm.motion.statutory_instrument_number,
        ) {
            // The Statutory Instrument API only goes back to the 13th of June 2017, so any SIs older than 2017 aren't able to be found.
            (Some(year), Some(number)) => {
                year > 2016 && if year == 2017 { number > 631 } else { true }
            }
            (Some(year), _) => year > 2016,
            _ => true,
        };

        if !is_prayer || !is_within_dataset {
            return Ok(None);
        }

        let potential_sis = match (
            edm.motion.statutory_instrument_number,
            edm.motion
                .statutory_instrument_year
                .clone()
                .map(|year| year.parse::<i32>().ok())
                .flatten(),
        ) {
            (Some(number), Some(year)) => {
                self.sis
                    .find(doc! {
                        "item.paper.paper_year": year,
                        "item.paper.paper_number": number,
                        "$or": [
                            {
                                "procedure.name": "Draft negative",
                            },
                            {
                                "procedure.name": "Made negative",
                            },
                        ],
                    })
                    .await?
                    .try_collect::<Vec<_>>()
                    .await?
            }
            _ => Vec::new(),
        };

        let potential_sis = match edm.motion.statutory_instrument_title.clone() {
            Some(title) if potential_sis.len() != 1 => {
                self.sis
                    .find(doc! {
                        "item.name": {
                            "$regex": regex::escape(&title),
                            "$options": "i",
                        },
                        "$or": [
                            {
                                "procedure.name": "Draft negative",
                            },
                            {
                                "procedure.name": "Made negative",
                            },
                        ],
                    })
                    .await?
                    .try_collect::<Vec<_>>()
                    .await?
            }
            _ => potential_sis,
        };

        match potential_sis.len() {
            1 => Ok(Some(potential_sis[0].item._id.clone())),
            x => Err(TaskError::StatutoryInstrumentUnknown(
                edm.motion.id,
                x,
                edm.motion.praying_against_negative_statutory_instrument_id,
                edm.motion.statutory_instrument_number,
                edm.motion.statutory_instrument_title.clone(),
                edm.motion.statutory_instrument_year.clone(),
            )),
        }
    }

    pub async fn normalize_si_name_comparison(&self) -> Result<(), TaskError> {
        let edms = self
            .edms
            .find(doc! {})
            .await?
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .skip(50000);

        let sis = self
            .sis
            .find(doc! {})
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        for edm in edms {
            if let Some(si) = sis.iter().find(|si| {
                edm.item
                    .motion_text
                    .to_lowercase()
                    .split(" ")
                    .collect::<Vec<_>>()
                    .join("")
                    .contains(
                        &si.item
                            .name
                            .to_lowercase()
                            .split(" ")
                            .collect::<Vec<_>>()
                            .join(""),
                    )
            }) {
                self.edms
                    .update_one(
                        doc! {
                            "item._id": edm.item._id,
                            "$nor": [
                                {
                                    "extra.source_kind": SourceKind::NsiNameComparison as u32,
                                },
                            ],
                        },
                        doc! {
                            "$push": {
                                "extra.source_kind": SourceKind::NsiNameComparison as u32,
                            },
                        },
                    )
                    .await?;

                self.edms
                    .update_one(
                        doc! {
                            "item._id": edm.item._id,
                            "extra.opposes_statutory_instrument": null,
                        },
                        doc! {
                            "$set": {
                                "extra.opposes_statutory_instrument": si.item._id.clone(),
                            },
                        },
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
