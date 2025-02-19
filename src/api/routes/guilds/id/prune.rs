use chorus::types::{GuildPruneQuerySchema, GuildPruneResult, Snowflake};
use poem::{
    handler,
    IntoResponse,
    web::{Data, Json, Path, Query},
};
use sqlx::MySqlPool;

use crate::{
    database::entities::{Config, Guild, Role, User},
    errors::{Error, GuildError},
};

#[handler]
pub async fn prune_members_dry_run(
    Data(db): Data<&MySqlPool>,
    Data(authed_user): Data<&User>,
    Data(config): Data<&Config>,
    Path(guild_id): Path<Snowflake>,
    Query(query): Query<GuildPruneQuerySchema>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    let me = guild
        .get_member(db, authed_user.id)
        .await?
        .ok_or(Error::Guild(GuildError::MemberNotFound))?;

    let my_highest = async {
        let mut roles = vec![];
        for role_id in me.roles.iter() {
            let role = Role::get_by_id(db, *role_id)
                .await?
                .ok_or(Error::Guild(GuildError::InvalidRole))?;

            roles.push(role);
        }

        let highest = roles
            .iter()
            .max_by_key(|r| r.position)
            .map(|r| r.position)
            .unwrap_or(0);
        <Result<u16, Error>>::Ok(highest)
    }
    .await?;

    let members = guild
        .calculate_inactive_members(db, query.days, query.include_roles, my_highest)
        .await?;

    Ok(Json(GuildPruneResult {
        pruned: Some(members.len()),
    }))
}

#[handler]
pub async fn prune_members(
    Data(db): Data<&MySqlPool>,
    Data(authed_user): Data<&User>,
    Data(config): Data<&Config>,
    Path(guild_id): Path<Snowflake>,
    Query(query): Query<GuildPruneQuerySchema>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    let me = guild
        .get_member(db, authed_user.id)
        .await?
        .ok_or(Error::Guild(GuildError::MemberNotFound))?;

    let my_highest = async {
        let mut roles = vec![];
        for role_id in &me.roles {
            let role = Role::get_by_id(db, *role_id)
                .await?
                .ok_or(Error::Guild(GuildError::InvalidRole))?;

            roles.push(role);
        }

        let highest = roles
            .iter()
            .max_by_key(|r| r.position)
            .map(|r| r.position)
            .unwrap_or(0);
        <Result<u16, Error>>::Ok(highest)
    }
    .await?;

    let members = guild
        .calculate_inactive_members(db, query.days, query.include_roles, my_highest)
        .await?;

    let total_count = members.len();
    for member in members {
        // TODO: Emit events?  Maybe write a special query for this?
        member.delete(db).await?;
    }

    Ok(Json(GuildPruneResult {
        pruned: if query.compute_prune_count.unwrap_or_default() {
            Some(total_count)
        } else {
            None
        },
    }))
}
