use bson::{DateTime, Document, doc};
use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::data::{
    DatasetItem,
    edm::{EDM_COLLECTION, EarlyDayMotion, EarlyDayMotionExtra, MotionKind},
    member::{MEMBER_COLLECTION, Member},
    party::{PARTY_COLLECTION, Party},
};

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct QueryItem {
    #[serde(flatten)]
    pub item: DatasetItem<EarlyDayMotion, EarlyDayMotionExtra>,
    pub signatures: Vec<Signature>,
    pub result: bson::Bson,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature {
    pub member: Member,
    pub party: Party,
    pub creation_date: DateTime,
    pub withdrawal_date: Option<DateTime>,
    pub sponsoring_order: Option<u32>,
}

pub trait Query {
    fn build(&self) -> Vec<Document>;
}

impl<Q: Query> Query for &Q {
    fn build(&self) -> Vec<Document> {
        (*self).build()
    }
}

impl Query for Document {
    fn build(&self) -> Vec<Document> {
        vec![self.clone()]
    }
}

impl<Q: Query> Query for Vec<Q> {
    fn build(&self) -> Vec<Document> {
        self.iter().flat_map(|query| query.build()).collect()
    }
}

pub trait DocumentExt: Sized {
    fn doc_add<Q: Query>(self, query: Q) -> Vec<Document>;
}

impl<Q2: Query> DocumentExt for Q2 {
    fn doc_add<Q: Query>(self, query: Q) -> Vec<Document> {
        let mut value = self.build();

        value.extend(query.build());

        value
    }
}

#[derive(Default, From)]
pub struct GroupQuery {
    pub prepend: Vec<Document>,
    pub groups: Vec<Vec<Document>>,
    pub append: Vec<Document>,
}

impl GroupQuery {
    pub fn prepend<Q: Query>(mut self, value: Q) -> Self {
        self.prepend.extend(value.build());

        self
    }

    pub fn with<Q: Query>(mut self, value: Q) -> Self {
        let mut groups = self.prepend.clone();

        groups.extend(value.build());

        groups.extend(self.append.clone());

        self.groups.push(groups);

        self
    }

    pub fn append<Q: Query>(mut self, value: Q) -> Self {
        self.append.extend(value.build());

        self
    }
}

impl Query for GroupQuery {
    fn build(&self) -> Vec<Document> {
        self.groups
            .clone()
            .into_iter()
            .flat_map(|queries| queries)
            .collect()
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetEarlyDayMotionData {}

impl SetEarlyDayMotionData {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Query for SetEarlyDayMotionData {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "result.primary_sponsor": {
                    "$getField": {
                        "input": {
                            "$arrayElemAt": [
                                {
                                    "$filter": {
                                        "input": "$signatures_copy",
                                        "as": "signature",
                                        "cond": {
                                            "$eq": [
                                                "$$signature.sponsoring_order",
                                                1,
                                            ],
                                        },
                                    },
                                },
                                0,
                            ],
                        },
                        "field": "display_name",
                    },
                },
                "result.id": "$item._id",
                "result.title": "$item.title",
                "result.text": "$item.motion_text",
                "result.date_tabled": "$item.date_tabled",
            },
        }]
    }
}

#[derive(Clone, Debug, Default, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CollectSignatures {}

impl CollectSignatures {
    pub fn new() -> Self {
        Self {}
    }
}

impl Query for CollectSignatures {
    fn build(&self) -> Vec<Document> {
        vec![
            doc! {
                "$lookup": {
                    "from": EDM_COLLECTION,
                    "localField": "item._id",
                    "foreignField": "item.amends.motion_id",
                    "as": "edms",
                },
            },
            doc! {
                "$unwind": {
                    "path": "$edms",
                    "preserveNullAndEmptyArrays": true,
                },
            },
            doc! {
                "$group": {
                    "_id": "$_id",
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "signatures": {
                        "$concatArrays": {
                            "$ifNull": [
                                "$edms.item.supporting_signatures",
                                [],
                            ],
                        },
                    },
                    "signatures": {
                        "$concatArrays": {
                            "$ifNull": [
                                "$edms.item.withdrawn_signatures",
                                [],
                            ],
                        },
                    },
                },
            },
            doc! {
                "$set": {
                    "signatures": {
                        "$setUnion": [
                            {
                                "$ifNull": [
                                    "$item.supporting_signatures",
                                    [],
                                ],
                            },
                            {
                                "$ifNull": [
                                    "$item.withdrawn_signatures",
                                    [],
                                ],
                            },
                            "$signatures",
                        ],
                    },
                },
            },
            doc! {
                "$lookup": {
                    "from": MEMBER_COLLECTION,
                    "localField": "signatures.member_id",
                    "foreignField": "item._id",
                    "as": "members",
                },
            },
            doc! {
                "$set": {
                    "members.item.edms": [],
                },
            },
            doc! {
                "$lookup": {
                    "from": PARTY_COLLECTION,
                    "localField": "members.item.party_id",
                    "foreignField": "item._id",
                    "as": "parties",
                },
            },
            doc! {
                "$set": {
                    "signatures": {
                        "$map": {
                            "input": "$members",
                            "as": "member",
                            "in": {
                                "member": "$$member.item",
                                "party": {
                                    "$getField": {
                                        "input": {
                                            "$arrayElemAt": [
                                                {
                                                    "$filter": {
                                                        "input": "$parties",
                                                        "as": "party",
                                                        "cond": {
                                                            "$eq": [
                                                                "$$party.item._id",
                                                                "$$member.item.party_id",
                                                            ],
                                                        },
                                                    },
                                                },
                                                0,
                                            ],
                                        },
                                        "field": "item",
                                    },
                                },
                                "creation_date": {
                                    "$getField": {
                                        "input": {
                                            "$arrayElemAt": [
                                                {
                                                    "$filter": {
                                                        "input": "$signatures",
                                                        "as": "signature",
                                                        "cond": {
                                                            "$eq": [
                                                                "$$signature.member_id",
                                                                "$$member.item._id",
                                                            ],
                                                        },
                                                    },
                                                },
                                                0,
                                            ],
                                        },
                                        "field": "creation_date",
                                    },
                                },
                                "withdrawal_date": {
                                    "$getField": {
                                        "input": {
                                            "$arrayElemAt": [
                                                {
                                                    "$filter": {
                                                        "input": "$signatures",
                                                        "as": "signature",
                                                        "cond": {
                                                            "$eq": [
                                                                "$$signature.member_id",
                                                                "$$member.item._id",
                                                            ],
                                                        },
                                                    },
                                                },
                                                0,
                                            ],
                                        },
                                        "field": "withdrawal_date",
                                    },
                                },
                                "sponsoring_order": {
                                    "$getField": {
                                        "input": {
                                            "$arrayElemAt": [
                                                {
                                                    "$filter": {
                                                        "input": "$signatures",
                                                        "as": "signature",
                                                        "cond": {
                                                            "$eq": [
                                                                "$$signature.member_id",
                                                                "$$member.item._id",
                                                            ],
                                                        },
                                                    },
                                                },
                                                0,
                                            ],
                                        },
                                        "field": "sponsoring_order",
                                    },
                                },
                            },
                        },
                    },
                },
            },
            doc! {
                "$set": {
                    "signatures_copy": "$signatures",
                },
            },
            doc! {
                "$project": {
                    "edms": 0,
                    "members": 0,
                    "parties": 0,
                },
            },
        ]
    }
}

#[derive(Clone, Debug, Default, Hash, From, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResetSignatures {}

impl ResetSignatures {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Query for ResetSignatures {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "signatures": "$signatures_copy",
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetSignatureCount {
    pub result: String,
}

impl SetSignatureCount {
    pub fn new(result: impl Into<String>) -> Self {
        Self {
            result: result.into(),
        }
    }
}

impl Query for SetSignatureCount {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "result.".to_owned() + &self.result: {
                    "$size": "$signatures",
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetSignaturesByDate {
    pub result: String,
    /// Whether to use the creation date, or the withdrawal date.
    pub creation: bool,
}

impl SetSignaturesByDate {
    pub fn new(result: impl Into<String>, creation: bool) -> Self {
        Self {
            result: result.into(),
            creation,
        }
    }
}

impl Query for SetSignaturesByDate {
    fn build(&self) -> Vec<Document> {
        vec![
            doc! {
                "$unwind": {
                    "path": "$signatures",
                    "preserveNullAndEmptyArrays": true,
                },
            },
            doc! {
                "$group": {
                    "_id": {
                        "id": "$_id",
                        "date": {
                            "$dateToString": {
                                "date": if self.creation { "$signatures.creation_date" } else { "$signatures.withdrawal_date" },
                                "format": "%Y-%m-%d",
                            },
                        },
                    },
                    "id": {
                        "$first": "$_id",
                    },
                    "date": {
                        "$first": {
                            "$dateToString": {
                                "date": if self.creation { "$signatures.creation_date" } else { "$signatures.withdrawal_date" },
                                "format": "%Y-%m-%d",
                            },
                        },
                    },
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "result": {
                        "$first": "$result",
                    },
                    "signatures_copy": {
                        "$first": "$signatures_copy",
                    },
                    "temp": {
                        "$sum": 1,
                    },
                },
            },
            doc! {
                "$group": {
                    "_id": "$id",
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "result": {
                        "$first": "$result",
                    },
                    "signatures_copy": {
                        "$first": "$signatures_copy",
                    },
                    "temp": {
                        "$push": {
                            "k": "$date",
                            "v": "$temp",
                        },
                    },
                },
            },
            doc! {
                "$set": {
                    "result.".to_owned() + &self.result: {
                        "$arrayToObject": {
                            "$sortArray": {
                                "input": {
                                    "$filter": {
                                        "input": "$temp",
                                        "as": "temp",
                                        "cond": {
                                            "$ne": [
                                                "$$temp.k",
                                                null,
                                            ],
                                        },
                                    },
                                },
                                "sortBy": {
                                    "k": 1,
                                },
                            },
                        },
                    },
                },
            },
        ]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetSignaturesByParty {
    pub result: String,
}

impl SetSignaturesByParty {
    pub fn new(result: impl Into<String>) -> Self {
        Self {
            result: result.into(),
        }
    }
}

impl Query for SetSignaturesByParty {
    fn build(&self) -> Vec<Document> {
        vec![
            doc! {
                "$unwind": {
                    "path": "$signatures",
                    "preserveNullAndEmptyArrays": true,
                },
            },
            doc! {
                "$group": {
                    "_id": {
                        "id": "$_id",
                        "party": "$signatures.party.name",
                    },
                    "id": {
                        "$first": "$_id",
                    },
                    "party": {
                        "$first": "$signatures.party.name",
                    },
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "result": {
                        "$first": "$result",
                    },
                    "signatures_copy": {
                        "$first": "$signatures_copy",
                    },
                    "temp": {
                        "$sum": 1,
                    },
                },
            },
            doc! {
                "$group": {
                    "_id": "$id",
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "result": {
                        "$first": "$result",
                    },
                    "signatures_copy": {
                        "$first": "$signatures_copy",
                    },
                    "temp": {
                        "$push": {
                            "k": "$party",
                            "v": "$temp",
                        },
                    },
                },
            },
            doc! {
                "$set": {
                    "result.".to_owned() + &self.result: {
                        "$arrayToObject": {
                            "$sortArray": {
                                "input": {
                                    "$filter": {
                                        "input": "$temp",
                                        "as": "temp",
                                        "cond": {
                                            "$ne": [
                                                "$$temp.k",
                                                null,
                                            ],
                                        },
                                    },
                                },
                                "sortBy": {
                                    "v": -1,
                                },
                            },
                        },
                    },
                },
            },
        ]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetSignaturesByMembership {
    pub result: String,
    /// Whether to use the initial membership date, or the most recent.
    pub initial: bool,
}

impl SetSignaturesByMembership {
    pub fn new(result: impl Into<String>, initial: bool) -> Self {
        Self {
            result: result.into(),
            initial,
        }
    }
}

impl Query for SetSignaturesByMembership {
    fn build(&self) -> Vec<Document> {
        vec![
            doc! {
                "$unwind": {
                    "path": "$signatures",
                    "preserveNullAndEmptyArrays": true,
                },
            },
            doc! {
                "$set": {
                    "temp": {
                        "$arrayElemAt": [
                            {
                                "$sortArray": {
                                    "input": {
                                        "$ifNull": [
                                            "$signatures.member.biography.house_memberships",
                                            [],
                                        ],
                                    },
                                    "sortBy": {
                                        "start_date": if self.initial {
                                            1
                                        } else {
                                            -1
                                        },
                                    },
                                },
                            },
                            0,
                        ],
                    },
                },
            },
            doc! {
                "$group": {
                    "_id": {
                        "id": "$_id",
                        "year": {
                            "$year": {
                                "$getField": {
                                    "input": "$temp",
                                    "field": "start_date",
                                },
                            },
                        },
                    },
                    "id": {
                        "$first": "$_id",
                    },
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "result": {
                        "$first": "$result",
                    },
                    "signatures": {
                        "$first": "$signatures",
                    },
                    "signatures_copy": {
                        "$first": "$signatures_copy",
                    },
                    "temp": {
                        "$sum": 1,
                    },
                    "year": {
                        "$first": {
                            "$toString": {
                                "$year": {
                                    "$getField": {
                                        "input": "$temp",
                                        "field": "start_date",
                                    },
                                },
                            },
                        },
                    },
                },
            },
            doc! {
                "$group": {
                    "_id": "$id",
                    "item": {
                        "$first": "$item",
                    },
                    "extra": {
                        "$first": "$extra",
                    },
                    "result": {
                        "$first": "$result",
                    },
                    "signatures_copy": {
                        "$first": "$signatures_copy",
                    },
                    "temp": {
                        "$push": {
                            "k": "$year",
                            "v": "$temp",
                        },
                    },
                },
            },
            doc! {
                "$set": {
                    "result.".to_owned() + &self.result: {
                        "$arrayToObject": {
                            "$sortArray": {
                                "input": {
                                    "$filter": {
                                        "input": "$temp",
                                        "as": "temp",
                                        "cond": {
                                            "$ne": [
                                                "$$temp.k",
                                                null,
                                            ],
                                        },
                                    },
                                },
                                "sortBy": {
                                    "k": 1,
                                },
                            },
                        },
                    },
                },
            },
            doc! {
                "$project": {
                    "temp": 0,
                }
            },
        ]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterSignaturesByBench {
    pub backbench: bool,
    pub government: Option<bool>,
}

impl FilterSignaturesByBench {
    pub const fn new(backbench: bool, government: Option<bool>) -> Self {
        Self {
            backbench,
            government,
        }
    }
}

impl Query for FilterSignaturesByBench {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "signatures": {
                    "$filter": {
                        "input": "$signatures",
                        "as": "signature",
                        "cond": {
                            if self.backbench {
                                "$eq"
                            } else {
                                "$gt"
                            }: [
                                {
                                    "$size": {
                                        "$filter": {
                                            "input": {
                                                "$setUnion": match self.government {
                                                    None => {
                                                        vec![
                                                            "$$signature.member.biography.government_posts",
                                                            "$$signature.member.biography.opposition_posts",
                                                        ]
                                                    },
                                                    Some(true) => vec!["$$signature.member.biography.government_posts"],
                                                    Some(false) => vec!["$$signature.member.biography.opposition_posts"],
                                                },
                                            },
                                            "as": "post",
                                            "cond": {
                                                "$and": [
                                                    {
                                                        "$gte": [
                                                            "$$signature.creation_date",
                                                            "$$post.start_date",
                                                        ],
                                                    },
                                                    {
                                                        "$cond": {
                                                            "if": {
                                                                "$eq": [
                                                                    "$$post.end_date",
                                                                    null,
                                                                ],
                                                            },
                                                            "then": true,
                                                            "else": {
                                                                "$lte": [
                                                                    "$$signature.creation_date",
                                                                    "$$post.end_date",
                                                                ],
                                                            },
                                                        },
                                                    },
                                                ],
                                            },
                                        },
                                    },
                                },
                                0,
                            ],
                        },
                    },
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterSignaturesByGender {
    pub genders: Vec<String>,
}

impl FilterSignaturesByGender {
    pub const fn new(genders: Vec<String>) -> Self {
        Self { genders }
    }
}

impl Query for FilterSignaturesByGender {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "signatures": {
                    "$filter": {
                        "input": "$signatures",
                        "as": "signature",
                        "cond": {
                            "$in": [
                                "$$signature.member.gender",
                                self.genders.clone(),
                            ],
                        },
                    },
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterByKeywords {}

impl FilterByKeywords {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Query for FilterByKeywords {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$match": {
                "item.motion_text": {
                    "$regex": "be annulled|be revoked|be not made|be not submitted|be withdrawn|be disapproved|does not approve|[ .,;:/\\(]s[ \\.,;:/]{0,3}[ir][ \\.,;:/\\)][ \\.,;:/n0-9\\)][ \\.,;:/n0-9\\)]{2}|statutory instrument|secondary legislation",
                    "$options": "i",
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterByPrimarySponsorBench {
    pub backbench: bool,
    pub government: Option<bool>,
}

impl FilterByPrimarySponsorBench {
    pub const fn new(backbench: bool, government: Option<bool>) -> Self {
        Self {
            backbench,
            government,
        }
    }
}

impl Query for FilterByPrimarySponsorBench {
    fn build(&self) -> Vec<Document> {
        let mut doc = FilterSignaturesByBench::new(self.backbench, self.government).build();

        doc.extend(vec![doc! {
            "$match": {
                "signatures.sponsoring_order": {
                    "$eq": 1,
                },
            },
        }]);

        doc
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterByMotionKind {
    pub motion_kinds: Vec<MotionKind>,
}

impl FilterByMotionKind {
    pub const fn new(motion_kinds: Vec<MotionKind>) -> Self {
        Self { motion_kinds }
    }
}

impl Query for FilterByMotionKind {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$match": {
                "extra.motion_kind": {
                    "$in": self.motion_kinds.iter().map(|motion_kind| motion_kind.clone() as i32).collect::<Vec<_>>(),
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterByDate {
    pub start_date: Option<DateTime>,
    pub end_date: Option<DateTime>,
}

impl FilterByDate {
    pub const fn new(start_date: Option<DateTime>, end_date: Option<DateTime>) -> Self {
        Self {
            start_date,
            end_date,
        }
    }

    pub fn new_by_year(start_year: i32, end_year: i32) -> Self {
        Self {
            start_date: Some(
                DateTime::builder()
                    .year(start_year)
                    .month(1)
                    .day(1)
                    .build()
                    .unwrap(),
            ),
            end_date: Some(
                DateTime::builder()
                    .year(end_year)
                    .month(1)
                    .day(1)
                    .build()
                    .unwrap(),
            ),
        }
    }
}

impl Query for FilterByDate {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$match": {
                "item.date_tabled": match (self.start_date, self.end_date) {
                    (Some(start_date), Some(end_date)) => doc! {
                        "$gte": start_date,
                        "$lte": end_date,
                    },
                    (Some(start_date), None) => doc! {
                        "$gte": start_date,
                    },
                    (None, Some(end_date)) => doc! {
                        "$lte": end_date,
                    },
                    (None, None) => doc! {
                        "$ne": null,
                    },
                }
            }
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterSignaturesByParty {
    pub parties: Vec<String>,
}

impl FilterSignaturesByParty {
    pub const fn new(parties: Vec<String>) -> Self {
        Self { parties }
    }
}

impl Query for FilterSignaturesByParty {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "signatures": {
                    "$filter": {
                        "input": "$signatures",
                        "as": "signature",
                        "cond": {
                            "$in": [
                                "$$signature.party.name",
                                self.parties.clone(),
                            ],
                        },
                    },
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilterSignaturesBySupporting {
    pub supporting: bool,
}

impl FilterSignaturesBySupporting {
    pub const fn new(supporting: bool) -> Self {
        Self { supporting }
    }
}

impl Query for FilterSignaturesBySupporting {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$set": {
                "signatures": {
                    "$filter": {
                        "input": {
                            "$ifNull": [
                                "$signatures",
                                [],
                            ],
                        },
                        "as": "signature",
                        "cond": {
                            if self.supporting {
                                "$eq"
                            } else {
                                "$ne"
                            }: [
                                "$$signature.withdrawal_date",
                                null,
                            ],
                        },
                    },
                },
            },
        }]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortBySignatureCount {
    pub ascending: bool,
}

impl SortBySignatureCount {
    pub const fn new(ascending: bool) -> Self {
        Self { ascending }
    }
}

impl Query for SortBySignatureCount {
    fn build(&self) -> Vec<Document> {
        vec![
            doc! {
                "$set": {
                    "temp": {
                        "$size": "$signatures",
                    },
                },
            },
            doc! {
                "$sort": {
                    "temp": if self.ascending {
                        1
                    } else {
                        -1
                    },
                },
            },
        ]
    }
}

#[derive(Clone, Debug, From, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortByDateTabled {
    pub ascending: bool,
}

impl SortByDateTabled {
    pub const fn new(ascending: bool) -> Self {
        Self { ascending }
    }
}

impl Query for SortByDateTabled {
    fn build(&self) -> Vec<Document> {
        vec![doc! {
            "$sort": {
                "item.date_tabled": if self.ascending {
                    1
                } else {
                    -1
                },
            },
        }]
    }
}
