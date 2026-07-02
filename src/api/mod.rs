pub mod edm;
pub mod member;
pub mod si;

use derive_more::{Display, Error, From};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum FlatResult<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Debug, Deserialize, Serialize, Display, Error, From)]
#[serde(untagged)]
pub enum ApiError {
    #[serde(skip)]
    ReqwestError(reqwest::Error),
    #[serde(skip)]
    WrapperDeserializeError(serde_json::Error),
    #[display("Failed to deserialize: {_0}, with content: {_1}")]
    #[serde(skip)]
    ResponseDeserializeError(serde_json::Error, String),
    PagedError(PagedResponse),
    DetailedErrorResponse(DetailedErrorResponse),
    String(#[error(not(source))] String),
    #[display("()")]
    None(#[error(not(source))] ()),
}

#[derive(Clone, Debug, Deserialize, Serialize, From, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum ApiResponse {
    PagedResponse(PagedResponse),
    ValueWithLinksResponse(ValueWithLinksResponse<Value>),
    ItemsResponse(ItemsResponse),
}

impl ApiResponse {
    pub fn try_unwrap<R: DeserializeOwned>(self) -> Result<R, ApiError> {
        match self {
            Self::PagedResponse(response) => {
                if !response.success || !response.errors.is_empty() || response.response == None {
                    return Err(ApiError::PagedError(response));
                }

                let response = match response.response {
                    Some(response) => response,
                    None => return Err(ApiError::PagedError(response)),
                };

                let text = serde_json::to_string(&response)?;

                Ok(match serde_json::from_value(response) {
                    Ok(res) => res,
                    Err(e) => return Err(ApiError::ResponseDeserializeError(e, text)),
                })
            }
            Self::ValueWithLinksResponse(response) => {
                let text = serde_json::to_string(&response.value)?;

                Ok(match serde_json::from_value(response.value) {
                    Ok(res) => res,
                    Err(e) => return Err(ApiError::ResponseDeserializeError(e, text)),
                })
            }
            Self::ItemsResponse(response) => {
                let items = response
                    .items
                    .into_iter()
                    .map(|item| item.value)
                    .collect::<Vec<_>>();
                let text = serde_json::to_string(&items)?;

                Ok(match serde_json::from_value(Value::Array(items)) {
                    Ok(res) => res,
                    Err(e) => return Err(ApiError::ResponseDeserializeError(e, text)),
                })
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Display, Error, Hash, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
#[display("{:#?}", self)]
pub struct PagedResponse {
    pub paging_info: Option<PagingInfo>,
    pub status_code: i32,
    pub success: bool,
    pub errors: Vec<String>,
    pub response: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub struct PagingInfo {
    pub format: Option<i32>,
    pub take: i32,
    pub total: i32,
    pub global_total: i32,
    pub status_counts: Vec<StatusCount>,
    pub global_status_counts: Vec<StatusCount>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub struct StatusCount {
    pub status_id: i32,
    pub count: i32,
}

#[derive(
    Clone, Debug, Deserialize, Serialize, Display, Error, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "camelCase")]
#[display("{:#?}", self)]
pub struct DetailedErrorResponse {
    pub r#type: Option<String>,
    pub title: Option<String>,
    pub status: Option<u32>,
    pub detail: Option<String>,
    pub instance: Option<String>,
    pub additional_prop1: Option<String>,
    pub additional_prop2: Option<String>,
    pub additional_prop3: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ItemsResponse {
    pub items: Vec<ValueWithLinksResponse<Value>>,
    pub total_results: i32,
    pub result_context: Option<String>,
    pub items_per_page: Option<i32>,
    pub links: Vec<LinkResponse>,
    pub result_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct ValueWithLinksResponse<T> {
    pub value: T,
    pub links: Vec<LinkResponse>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct LinkResponse {
    pub rel: String,
    pub href: String,
    pub method: String,
}

pub trait ApiRequest {
    type Response: DeserializeOwned + Serialize;

    fn url(&self) -> impl Into<String>;

    async fn get(&self) -> Result<Self::Response, ApiError>;

    async fn get_response(&self) -> Result<Self::Response, ApiError> {
        let wrapper: FlatResult<ApiResponse, ApiError> = Client::new()
            .get(self.url().into())
            .header("Accept", "application/json")
            .send()
            .await?
            .json()
            .await?;

        let wrapper = match wrapper {
            FlatResult::Ok(wrapper) => wrapper,
            FlatResult::Err(error) => return Err(error.into()),
        };

        wrapper.try_unwrap()
    }
}
