use axum::{Extension, Json, extract::State, http::StatusCode};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::AppError,
    models::{
        app_state::AppState,
        joint::{
            CreateJointRequest, CreateJointResponse, JoinJointRequest, JoinJointResponse, Joint,
            JointWithDistance, ListJointsRequest, ListJointsResponse,
        },
    },
};

/// Create a new joint
#[utoipa::path(
    post,
    path = "/api/v1/joints",
    request_body = CreateJointRequest,
    responses(
        (status = 201, description = "Joint created successfully", body = CreateJointResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Joints",
    security(("bearer" = []))
)]
pub async fn create_joint(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>, // TODO: Will add auth middleware later
    Json(payload): Json<CreateJointRequest>,
) -> Result<(StatusCode, Json<CreateJointResponse>), AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Calculate expiration time if provided
    let expires_at = payload
        .expires_in_hours
        .map(|hours| chrono::Utc::now() + chrono::Duration::hours(hours as i64));

    // Insert joint
    let joint = sqlx::query_as!(
        Joint,
        r#"
        INSERT INTO joints (name, description, latitude, longitude, radius, expires_at, creator_id, joint_type, visibility )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, name, creator_id, joint_type, visibility, latitude, longitude, radius, created_at, expires_at, description, is_active"#,
        payload.name,
        payload.description,
        payload.latitude,
        payload.longitude,
        payload.radius,
        expires_at,
        user_id,
        payload.joint_type.unwrap_or_else(|| "public".to_string()),
        payload.visibility.unwrap_or_else(|| "visible".to_string())
    )
    .fetch_one(&state.db)
    .await?;

    // Auto-join creator as a member with 'creator' role
    sqlx::query!(
        r#"
        INSERT INTO joint_members (joint_id, user_id, role)
        VALUES ($1, $2, 'creator')
        "#,
        joint.id,
        user_id
    )
    .execute(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateJointResponse {
            joint,
            message: "Joint created successfully!".to_string(),
        }),
    ))
}

/// List nearby joints
#[utoipa::path(
    post,
    path = "/api/v1/joints/nearby",
    request_body = ListJointsRequest,
    responses(
        (status = 200, description = "List of nearby joints", body = ListJointsResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Joints"
)]
pub async fn list_nearby_joints(
    State(state): State<AppState>,
    Json(payload): Json<ListJointsRequest>,
) -> Result<Json<ListJointsResponse>, AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Query nearby joints using Haversine formula
    // Note: For production, consider using PostGIS for better performance
    let joints = sqlx::query!(
        r#"
        WITH distances AS (
            SELECT 
                j.*,
                (6371000 * acos(
                    cos(radians($1)) * cos(radians(j.latitude)) *
                    cos(radians(j.longitude) - radians($2)) +
                    sin(radians($1)) * sin(radians(j.latitude))
                )) as distance_meters,
                COUNT(jm.id) as member_count
            FROM joints j
            LEFT JOIN joint_members jm ON j.id = jm.joint_id
            WHERE j.is_active = true
            AND (j.expires_at IS NULL OR j.expires_at > NOW())
            GROUP BY j.id
        )
        SELECT 
            id, name, description, latitude, longitude, radius, 
            is_active, expires_at, creator_id, created_at, joint_type, visibility,
            distance_meters, member_count
        FROM distances
        WHERE distance_meters <= $3
        ORDER BY distance_meters ASC
        "#,
        payload.latitude,
        payload.longitude,
        payload.radius_meters as f64
    )
    .fetch_all(&state.db)
    .await?;

    let joints_with_distance: Vec<JointWithDistance> = joints
        .into_iter()
        .map(|row| JointWithDistance {
            joint: Joint {
                id: row.id,
                name: row.name,
                description: row.description,
                latitude: row.latitude,
                longitude: row.longitude,
                radius: row.radius,
                joint_type: row.joint_type,
                visibility: row.visibility,
                is_active: Some(row.is_active),
                expires_at: Some(row.expires_at),
                creator_id: row.creator_id,
                created_at: row.created_at,
            },
            distance_meters: row.distance_meters.unwrap_or(0.0),
            member_count: row.member_count.unwrap_or(0),
        })
        .collect();

    let count = joints_with_distance.len();

    Ok(Json(ListJointsResponse {
        joints: joints_with_distance,
        count,
    }))
}

/// Join a joint
#[utoipa::path(
    post,
    path = "/api/v1/joints/join",
    request_body = JoinJointRequest,
    responses(
        (status = 200, description = "Successfully joined joint", body = JoinJointResponse),
        (status = 400, description = "Invalid input or already a member"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Joint not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Joints",
    security(("bearer" = []))
)]
pub async fn join_joint(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<JoinJointRequest>,
) -> Result<Json<JoinJointResponse>, AppError> {
    // Check if joint exists and is active
    let _joint = sqlx::query_as!(
        Joint,
        "SELECT id, name, creator_id, joint_type, visibility, latitude, longitude, radius, created_at, expires_at, description, is_active FROM joints WHERE id = $1",
        payload.joint_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::ValidationError(
        "Joint not found or inactive".to_string(),
    ))?;

    // Check if already a member
    let existing_member = sqlx::query!(
        "SELECT id FROM joint_members WHERE joint_id = $1 AND user_id = $2",
        payload.joint_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    if existing_member.is_some() {
        return Err(AppError::ValidationError(
            "Already a member of this joint".to_string(),
        ));
    }

    // Add user as member
    sqlx::query!(
        r#"
        INSERT INTO joint_members (joint_id, user_id, role)
        VALUES ($1, $2, 'member')
        "#,
        payload.joint_id,
        user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(JoinJointResponse {
        message: "Successfully joined the joint!".to_string(),
        joined: true,
    }))
}

/// Get user's active joints
#[utoipa::path(
    get,
    path = "/api/v1/joints/active",
    responses(
        (status = 200, description = "List of user's active joints", body = ListJointsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Joints",
    security(("bearer" = []))
)]
pub async fn get_active_joints(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<ListJointsResponse>, AppError> {
    let joints = sqlx::query!(
        r#"
        SELECT
            j.id, j.name, j.description, j.latitude, j.longitude, j.radius,
            j.expires_at, j.creator_id, j.created_at, j.joint_type, j.visibility, j.is_active,
            COUNT(jm.id) as member_count
        FROM joints j
        INNER JOIN joint_members jm_user ON j.id = jm_user.joint_id AND jm_user.user_id = $1
        LEFT JOIN joint_members jm ON j.id = jm.joint_id
        WHERE j.expires_at > NOW()
        AND (j.is_active IS NULL OR j.is_active = true)
        GROUP BY j.id
        ORDER BY j.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    let joints_with_distance: Vec<JointWithDistance> = joints
        .into_iter()
        .map(|row| JointWithDistance {
            joint: Joint {
                id: row.id,
                name: row.name,
                creator_id: row.creator_id,
                joint_type: row.joint_type,
                visibility: row.visibility,
                latitude: row.latitude,
                longitude: row.longitude,
                radius: row.radius,
                created_at: row.created_at,
                expires_at: Some(row.expires_at),
                description: row.description,
                is_active: Some(row.is_active),
            },
            distance_meters: 0.0, // Not relevant for active joints
            member_count: row.member_count.unwrap_or(0),
        })
        .collect();

    let count = joints_with_distance.len();

    Ok(Json(ListJointsResponse {
        joints: joints_with_distance,
        count,
    }))
}

/// Leave a joint
#[utoipa::path(
    post,
    path = "/api/v1/joints/leave",
    request_body = JoinJointRequest,
    responses(
        (status = 200, description = "Successfully left joint", body = JoinJointResponse),
        (status = 400, description = "Invalid input or not a member"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Joint not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Joints",
    security(("bearer" = []))
)]
pub async fn leave_joint(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<JoinJointRequest>,
) -> Result<Json<JoinJointResponse>, AppError> {
    // Check if user is a member
    let member = sqlx::query!(
        "SELECT id, role FROM joint_members WHERE joint_id = $1 AND user_id = $2",
        payload.joint_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::ValidationError(
        "You are not a member of this joint".to_string(),
    ))?;

    // If user is creator, they cannot leave (must transfer admin or delete joint)
    if member.role == "creator" {
        return Err(AppError::ValidationError(
            "Creators cannot leave joints. Please transfer admin rights or delete the joint."
                .to_string(),
        ));
    }

    // Remove user from joint
    sqlx::query!(
        "DELETE FROM joint_members WHERE joint_id = $1 AND user_id = $2",
        payload.joint_id,
        user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(JoinJointResponse {
        message: "Successfully left the joint!".to_string(),
        joined: false,
    }))
}
