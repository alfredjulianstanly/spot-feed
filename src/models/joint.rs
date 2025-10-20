use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Joint (location-based group) from database
#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct Joint {
    pub id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub joint_type: String,
    pub visibility: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Create a new joint
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateJointRequest {
    /// Joint name (3-100 characters)
    #[validate(length(min = 3, max = 100))]
    #[schema(example = "Coffee Lovers Downtown")]
    pub name: String,

    /// Optional description
    #[validate(length(max = 500))]
    #[schema(example = "Meet fellow coffee enthusiasts!")]
    pub description: Option<String>,

    /// Latitude coordinate
    #[schema(example = 40.7128)]
    pub latitude: f64,

    /// Longitude coordinate
    #[schema(example = -74.0060)]
    pub longitude: f64,

    /// Radius in meters (10-5000)
    #[validate(range(min = 10, max = 5000))]
    #[schema(example = 500)]
    pub radius: i32,

    /// Expiration time in hours (1-6, default 6)
    #[validate(range(min = 1, max = 6))]
    #[schema(example = 6)]
    pub expires_in_hours: Option<i32>,

    /// Joint type (public/private)
    #[schema(example = "public")]
    pub joint_type: Option<String>,

    /// Visibility (visible/hidden)
    #[schema(example = "visible")]
    pub visibility: Option<String>,
}

/// Joint creation response
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateJointResponse {
    /// Created joint
    pub joint: Joint,
    /// Success message
    pub message: String,
}

/// List nearby joints request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListJointsRequest {
    /// User's current latitude
    #[schema(example = 40.7128)]
    pub latitude: f64,

    /// User's current longitude
    #[schema(example = -74.0060)]
    pub longitude: f64,

    /// Search radius in meters (max 10000)
    #[validate(range(min = 1, max = 10000))]
    #[schema(example = 1000)]
    pub radius_meters: i32,
}

/// List of joints response
#[derive(Debug, Serialize, ToSchema)]
pub struct ListJointsResponse {
    /// List of nearby joints
    pub joints: Vec<JointWithDistance>,
    /// Total count
    pub count: usize,
}

/// Joint with distance from user
#[derive(Debug, Serialize, ToSchema)]
pub struct JointWithDistance {
    #[serde(flatten)]
    pub joint: Joint,
    /// Distance in meters from user's location
    pub distance_meters: f64,
    /// Current member count
    pub member_count: i64,
}

/// Join a joint request
#[derive(Debug, Deserialize, ToSchema)]
pub struct JoinJointRequest {
    /// Joint ID to join
    pub joint_id: Uuid,
}

/// Join joint response
#[derive(Debug, Serialize, ToSchema)]
pub struct JoinJointResponse {
    /// Success message
    pub message: String,
    /// Whether join was successful
    pub joined: bool,
}
