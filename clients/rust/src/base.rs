use std::sync::Arc;

use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

pub(crate) struct BaseClient {
    client: Arc<Client>,
    address: String,
    api_key: Option<String>,
}

#[derive(Debug)]
pub enum APIErrorVariant {
    Network,
    MalformedResponse,
    Unauthorized,
    NotFound,
    BadClientData,
    UnexpectedStatusCode,
}
#[derive(Debug)]
pub struct APIError {
    pub variant: APIErrorVariant,
    pub message: String,
}
pub type APIResponse<T> = Result<T, APIError>;

impl BaseClient {
    pub fn new(address: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            address,
            api_key: None,
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    fn get_client(
        &self,
        method: Method,
        path: String,
        query: Option<Vec<(String, String)>>,
    ) -> RequestBuilder {
        let prefix = "/api/v1/";
        let url = format!("{}{}{}", self.address, prefix, path);
        let builder = match method {
            Method::GET => self.client.get(&url).query(&query.unwrap_or_default()),
            Method::POST => self.client.post(&url),
            Method::PUT => self.client.put(&url),
            Method::PATCH => self.client.patch(&url),
            Method::DELETE => self.client.delete(&url),
            _ => unimplemented!(),
        };

        if let Some(api_key) = &self.api_key {
            builder.header("x-api-key", api_key.clone())
        } else {
            builder
        }
    }

    async fn check_status_code(
        &self,
        res: Response,
        expected_status_code: StatusCode,
    ) -> Result<Response, APIError> {
        let status = res.status();
        if status != expected_status_code {
            let variant = match status {
                StatusCode::UNAUTHORIZED => APIErrorVariant::Unauthorized,
                StatusCode::NOT_FOUND => APIErrorVariant::NotFound,
                StatusCode::UNPROCESSABLE_ENTITY => APIErrorVariant::BadClientData,
                _ => APIErrorVariant::UnexpectedStatusCode,
            };
            return Err(APIError {
                variant,
                message: res.text().await.unwrap_or_default(),
            });
        }
        Ok(res)
    }

    async fn get_json_response<T: for<'de> Deserialize<'de>>(
        &self,
        res: Response,
    ) -> APIResponse<T> {
        res.json::<T>().await.map_err(|e| APIError {
            variant: APIErrorVariant::MalformedResponse,
            message: e.to_string(),
        })
    }

    async fn handle_api_response<T: for<'de> Deserialize<'de>>(
        &self,
        res: Response,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = self.check_status_code(res, expected_status_code).await?;
        self.get_json_response(res).await
    }

    fn network_error(&self) -> APIError {
        APIError {
            variant: APIErrorVariant::Network,
            message: "Network error. Please try again".into(),
        }
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        query_params: Option<Vec<(String, String)>>,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self
            .get_client(Method::GET, path, query_params)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(self.network_error()),
        };
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn delete<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self.get_client(Method::DELETE, path, None).send().await {
            Ok(res) => res,
            Err(_) => return Err(self.network_error()),
        };
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn delete_with_body<T: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        body: S,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self
            .get_client(Method::DELETE, path, None)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(self.network_error()),
        };
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn put<T: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        body: S,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self
            .get_client(Method::PUT, path, None)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(self.network_error()),
        };
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn patch<T: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        body: S,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self
            .get_client(Method::PATCH, path, None)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(self.network_error()),
        };
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn post<T: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        body: S,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self
            .get_client(Method::POST, path, None)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(self.network_error()),
        };

        self.handle_api_response(res, expected_status_code).await
    }
}
