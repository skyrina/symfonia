use chorus::types::{jwt::Claims, PermissionFlags, Snowflake};
use poem::{
    handler,
    http::StatusCode,
    IntoResponse,
    Response, web::{Data, Json, Path},
};
use sqlx::MySqlPool;

use crate::{
    database::entities::Guild,
    errors::{Error, GuildError},
};

#[handler]
pub async fn bulk_assign_roles(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path((guild_id, role_id)): Path<(Snowflake, Snowflake)>,
    Json(member_ids): Json<Vec<Snowflake>>,
) -> poem::Result<impl IntoResponse> {
    if role_id == guild_id {
        return Err(Error::Guild(GuildError::InvalidRole).into());
    }

    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    let authed_member = guild
        .get_member(db, claims.id)
        .await?
        .ok_or(Error::Guild(GuildError::MemberNotFound))?;

    if !authed_member
        .permissions
        .contains(PermissionFlags::MANAGE_ROLES)
    {
        return Err(Error::Guild(GuildError::InsufficientPermissions).into());
    }

    guild
        .get_role(db, role_id)
        .await?
        .ok_or(Error::Guild(GuildError::RoleNotFound))?;

    for member_id in member_ids {
        let mut member = guild
            .get_member(db, member_id)
            .await?
            .ok_or(Error::Guild(GuildError::MemberNotFound))?;
        member.populate_relations(db).await?;
        if member.roles.contains(&role_id) {
            member.remove_role(db, role_id).await?;
        } else {
            member.add_role(db, role_id).await?;
        }
    }

    Ok(Response::builder().status(StatusCode::NO_CONTENT).finish())
}
