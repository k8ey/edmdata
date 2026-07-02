use bson::DateTime;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    api::edm::{PublishedEarlyDayMotionDetails, PublishedEarlyDayMotionSponsor},
    data::{member::MemberID, si::StatutoryInstrumentID},
};

pub const EDM_COLLECTION: &str = "early-day-motions";

pub type EarlyDayMotionID = u32;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EarlyDayMotionExtra {
    pub opposes_statutory_instrument: Option<StatutoryInstrumentID>,
    pub motion_kind: Option<MotionKind>,
    pub source_kind: Vec<SourceKind>,
}

impl From<&PublishedEarlyDayMotionDetails> for EarlyDayMotionExtra {
    fn from(value: &PublishedEarlyDayMotionDetails) -> Self {
        Self {
            opposes_statutory_instrument: None,
            motion_kind: None,
            source_kind: if value
                .motion
                .praying_against_negative_statutory_instrument_id
                .is_some()
                || value.motion.statutory_instrument_number.is_some()
                || value.motion.statutory_instrument_title.is_some()
                || value.motion.statutory_instrument_year.is_some()
            {
                vec![SourceKind::Api]
            } else {
                Vec::new()
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EarlyDayMotion {
    pub _id: EarlyDayMotionID,
    pub title: String,
    pub motion_text: String,
    pub date_tabled: DateTime,
    pub status: EarlyDayMotionStatus,
    pub status_date: DateTime,
    pub supporting_signatures: Vec<EarlyDayMotionSignature>,
    pub withdrawn_signatures: Vec<EarlyDayMotionSignature>,
    pub amends: Option<EarlyDayMotionAmendment>,
    /// The EDM Number, stated as `EDM {uin}[A{amendment_suffix}]?`.
    ///
    /// This number resets to 1 after each Parliamentary session, and is incremented by 1 for each EDM created in that session.
    pub uin: u32,

    pub _updated: DateTime,
}

impl From<PublishedEarlyDayMotionDetails> for EarlyDayMotion {
    fn from(value: PublishedEarlyDayMotionDetails) -> Self {
        let (supporting_signatures, withdrawn_signatures) = value
            .sponsors
            .into_iter()
            .partition::<Vec<_>, _>(|signature| {
                !signature.is_withdrawn || signature.withdrawn_date.is_none()
            });
        let supporting_signatures = supporting_signatures
            .into_iter()
            .map(|signature| EarlyDayMotionSignature::from(signature))
            .collect();
        let withdrawn_signatures = withdrawn_signatures
            .into_iter()
            .map(|signature| EarlyDayMotionSignature::from(signature))
            .collect();

        EarlyDayMotion {
            _id: value.motion.id as EarlyDayMotionID,
            title: value.motion.title.unwrap_or(String::new()),
            motion_text: value.motion.motion_text.unwrap_or(String::new()),
            date_tabled: value.motion.date_tabled.and_utc().into(),
            status: (value.motion.status as u8).into(),
            status_date: value.motion.status_date.and_utc().into(),
            supporting_signatures,
            withdrawn_signatures,
            amends: match (
                value.motion.amendment_to_motion_id,
                value.motion.amendment_suffix,
            ) {
                (Some(motion_id), Some(amendment_suffix)) => Some(EarlyDayMotionAmendment {
                    motion_id: motion_id as EarlyDayMotionID,
                    amendment_suffix,
                }),
                _ => None,
            },
            uin: value.motion.uin as u32,
            _updated: DateTime::now(),
        }
    }
}

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EarlyDayMotionStatus {
    Published = 0,
    Withdrawn = 1,
}

impl From<u8> for EarlyDayMotionStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Published,
            1 => Self::Withdrawn,
            x => panic!("Unknown EarlyDayMotionStatus variant: ({x})"),
        }
    }
}

pub type EarlyDayMotionSignatureID = u32;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EarlyDayMotionSignature {
    pub _id: EarlyDayMotionSignatureID,
    pub member_id: MemberID,
    pub creation_date: DateTime,
    pub withdrawal_date: Option<DateTime>,
    pub sponsoring_order: Option<u32>,
}

impl From<PublishedEarlyDayMotionSponsor> for EarlyDayMotionSignature {
    fn from(value: PublishedEarlyDayMotionSponsor) -> Self {
        Self {
            _id: value.id as EarlyDayMotionSignatureID,
            member_id: value.member.mnis_id as MemberID,
            creation_date: value.created_when.and_utc().into(),
            withdrawal_date: value.withdrawn_date.map(|dt| dt.and_utc().into()),
            sponsoring_order: value.sponsoring_order.map(|x| x as u32),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EarlyDayMotionAmendment {
    /// EDM ID of the original EDM, not the amendment EDM.
    pub motion_id: EarlyDayMotionID,
    pub amendment_suffix: String,
}

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MotionKind {
    /// A fatal prayer motion, which objects to a negative statutory instrument becoming or continuing as law.
    ///
    /// Usually these are `be annulled` style prayer motions, though I would consider any motion with an intent to annul to be a fatal prayer motion.
    Fatal,
    /// A non-fatal prayer motion, which intends to bring debate around a negative statutory instrument, but cannot object to a negative statutory instrument becoming or continuing as law.
    ///
    /// I believe non-fatal motions are only able to be tabled by members of the House of Lords, I'm still going to use this terminology, because it makes most sense to me.
    NonFatal,
    /// This motion is potentially not actually a fatal prayer motion, since it does not include the `be annulled` statement,
    /// but still has the intent of a fatal prayer motion.
    SeemsFatal,
}

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SourceKind {
    List,
    Api,
    NsiNameComparison,
    ManualReview,
}
