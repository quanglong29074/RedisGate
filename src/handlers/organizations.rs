// Organization management handlers

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api_models::{
    ApiResponse, CreateOrganizationRequest, OrganizationResponse, PaginatedResponse,
    PaginationParams,
};
use crate::middleware::{AppState, CurrentUser};
use crate::models::Organization;

type ErrorResponse = (StatusCode, Json<ApiResponse<()>>);

// Helper function to convert Organization to OrganizationResponse
fn organization_to_response(organization: Organization) -> OrganizationResponse {
    OrganizationResponse {
        id: organization.id,
        name: organization.name,
        slug: organization.slug,
        description: organization.description,
        owner_id: organization.owner_id,
        is_active: organization.is_active.unwrap_or(true),
        plan: organization.plan.unwrap_or_else(|| "free".to_string()),
        max_redis_instances: organization.max_redis_instances.unwrap_or(3),
        max_api_keys: organization.max_api_keys.unwrap_or(10),
        created_at: organization.created_at.unwrap_or_else(|| Utc::now()),
        updated_at: organization.updated_at.unwrap_or_else(|| Utc::now()),
    }
}

pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<Json<ApiResponse<OrganizationResponse>>, ErrorResponse> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Check if organization slug is unique
    let existing_org = sqlx::query!(
        "SELECT id FROM organizations WHERE slug = $1",
        payload.slug
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    if existing_org.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Organization with this slug already exists".to_string())),
        ));
    }

    let org_id = Uuid::new_v4();
    let now = Utc::now();

    // Create organization
    sqlx::query!(
        r#"
        INSERT INTO organizations (id, name, slug, description, owner_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        org_id,
        payload.name,
        payload.slug,
        payload.description,
        current_user.id,
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to create organization: {}", e))),
        )
    })?;

    // Add user as owner in memberships
    let membership_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO organization_memberships (id, user_id, organization_id, role, permissions, joined_at, created_at, updated_at)
        VALUES ($1, $2, $3, 'owner', ARRAY['*'], $4, $5, $6)
        "#,
        membership_id,
        current_user.id,
        org_id,
        now,
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to create organization membership: {}", e))),
        )
    })?;

    // Fetch created organization
    let organization = sqlx::query_as!(
        Organization,
        "SELECT * FROM organizations WHERE id = $1",
        org_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to fetch created organization: {}", e))),
        )
    })?;

    let org_response = organization_to_response(organization);

    Ok(Json(ApiResponse::success(org_response)))
}

pub async fn list_organizations(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<OrganizationResponse>>>, ErrorResponse> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100); // Max 100 items per page
    let offset = (page - 1) * limit;

    // Get organizations where user is a member
    let organizations = sqlx::query_as!(
        Organization,
        r#"
        SELECT o.* FROM organizations o
        INNER JOIN organization_memberships om ON o.id = om.organization_id
        WHERE om.user_id = $1 AND om.is_active = true
        ORDER BY o.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        current_user.id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    // Get total count
    let total_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM organizations o
        INNER JOIN organization_memberships om ON o.id = om.organization_id
        WHERE om.user_id = $1 AND om.is_active = true
        "#,
        current_user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    let org_responses: Vec<OrganizationResponse> = organizations
        .into_iter()
        .map(organization_to_response)
        .collect();

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    let paginated_response = PaginatedResponse {
        items: org_responses,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(Json(ApiResponse::success(paginated_response)))
}

pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ApiResponse<OrganizationResponse>>, ErrorResponse> {
    // Check if user has access to this organization
    let organization = sqlx::query_as!(
        Organization,
        r#"
        SELECT o.* FROM organizations o
        INNER JOIN organization_memberships om ON o.id = om.organization_id
        WHERE o.id = $1 AND om.user_id = $2 AND om.is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    let org_response = organization_to_response(organization);

    Ok(Json(ApiResponse::success(org_response)))
}

pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateOrganizationRequest>, // Reusing the same request struct
) -> Result<Json<ApiResponse<OrganizationResponse>>, ErrorResponse> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Check if user is owner of this organization
    let org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    if org_membership.role != "owner" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Only organization owners can update organization details".to_string())),
        ));
    }

    // Check if new slug is unique (if changed)
    let existing_org = sqlx::query!(
        "SELECT id FROM organizations WHERE slug = $1 AND id != $2",
        payload.slug,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    if existing_org.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Organization with this slug already exists".to_string())),
        ));
    }

    let now = Utc::now();

    // Update organization
    sqlx::query!(
        r#"
        UPDATE organizations 
        SET name = $1, slug = $2, description = $3, updated_at = $4
        WHERE id = $5
        "#,
        payload.name,
        payload.slug,
        payload.description,
        now,
        org_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to update organization: {}", e))),
        )
    })?;

    // Fetch updated organization
    let organization = sqlx::query_as!(
        Organization,
        "SELECT * FROM organizations WHERE id = $1",
        org_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to fetch updated organization: {}", e))),
        )
    })?;

    let org_response = organization_to_response(organization);

    Ok(Json(ApiResponse::success(org_response)))
}

pub async fn delete_organization(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    // Check if user is owner of this organization
    let org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    if org_membership.role != "owner" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Only organization owners can delete the organization".to_string())),
        ));
    }

    // Check if organization has active Redis instances
    let active_instances = sqlx::query!(
        "SELECT COUNT(*) as count FROM redis_instances WHERE organization_id = $1 AND deleted_at IS NULL",
        org_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    if active_instances > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Cannot delete organization with active Redis instances".to_string())),
        ));
    }

    // Soft delete organization
    let now = Utc::now();
    sqlx::query!(
        "UPDATE organizations SET is_active = false, updated_at = $1 WHERE id = $2",
        now,
        org_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to delete organization: {}", e))),
        )
    })?;

    // Deactivate all memberships
    sqlx::query!(
        "UPDATE organization_memberships SET is_active = false, updated_at = $1 WHERE organization_id = $2",
        now,
        org_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to deactivate memberships: {}", e))),
        )
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Organization deleted successfully".to_string()),
        timestamp: Utc::now(),
    }))
}