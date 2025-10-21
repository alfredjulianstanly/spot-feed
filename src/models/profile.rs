use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// User profile
#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub profile_picture_url: Option<String>,
    pub phone_number: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Update profile request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 100))]
    #[schema(example = "John Doe")]
    pub display_name: Option<String>,

    #[schema(example = "https://example.com/avatar.jpg")]
    pub profile_picture_url: Option<String>,

    #[validate(length(max = 12))]
    #[schema(example = "+1234567890")]
    pub phone_number: Option<String>,
}

/// Profile response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub profile: UserProfile,
}
