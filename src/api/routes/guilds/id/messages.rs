use chorus::types::{MessageSearchQuery, MessageSearchResponse, PermissionFlags, Snowflake};
use poem::{
    handler,
    IntoResponse,
    web::{Data, Json, Path, Query},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    database::entities::{Guild, User},
    errors::{Error, GuildError},
};

#[handler]
pub async fn search(
    Data(db): Data<&MySqlPool>,
    Data(authed_user): Data<&User>,
    Path(guild_id): Path<Snowflake>,
    Query(payload): Query<MessageSearchQuery>,
) -> poem::Result<impl IntoResponse> {
    let limit = payload.limit.map(|x| x.min(100)).unwrap_or(50);

    if limit <= 0 {
        return Err(poem::error::Error::from_string(
            "limit must be between 1 and 100",
            poem::http::StatusCode::UNPROCESSABLE_ENTITY,
        ));
    }

    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    let authed_member = guild
        .get_member(db, authed_user.id)
        .await?
        .ok_or(Error::Guild(GuildError::MemberNotFound))?;

    if !authed_member
        .permissions
        .has_permission(PermissionFlags::VIEW_CHANNEL)
    {
        return Err(Error::Guild(GuildError::InsufficientPermissions).into());
    } else if !authed_member
        .permissions
        .has_permission(PermissionFlags::READ_MESSAGE_HISTORY)
    {
        return Ok(Json(MessageSearchResponse::default()).into_response());
    }

    todo!("unfinished search implementation");
    Ok(Json(json!({})).into_response())
}
