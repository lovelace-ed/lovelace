#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
/// A response from the API.
pub struct ApiResponse<D> {
    success: bool,
    data: Option<D>,
    error: Option<ApiError>,
}

impl<D> ApiResponse<D> {
    /// Create a new ok response.
    pub fn new_ok(data: D) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    /// Create a new error response.
    pub fn new_err<C>(reason: C) -> Self
    where
        C: Into<String>,
    {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                reason: reason.into(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
/// An error
pub struct ApiError {
    reason: String,
}

#[cfg(test)]
mod test_api_response {
    use super::{ApiError, ApiResponse};

    #[test]
    fn test_ok() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        pub struct Res {
            a: bool,
            b: String,
        }
        let res = ApiResponse::new_ok(Res {
            a: false,
            b: "true".to_string(),
        });
        let str = serde_json::to_string(&res).expect("failed to serialize");
        let res: ApiResponse<Res> = serde_json::from_str(&str).expect("failed to deserialize");
        assert_eq!(
            res,
            ApiResponse {
                success: true,
                data: Some(Res {
                    a: false,
                    b: "true".to_string(),
                }),
                error: None
            }
        );
    }
    #[test]
    fn test_err() {
        let res = ApiResponse::<()>::new_err("Some error");
        let str = serde_json::to_string(&res).expect("failed to serialize");
        let res: ApiResponse<()> = serde_json::from_str(&str).expect("failed to deserialize");
        assert_eq!(
            res,
            ApiResponse::<()> {
                success: false,
                error: Some(ApiError {
                    reason: "Some error".to_string()
                }),
                data: None
            }
        )
    }
}
