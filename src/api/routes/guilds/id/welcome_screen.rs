use chorus::types::{GuildModifyWelcomeScreenSchema, jwt::Claims, Snowflake, WelcomeScreenObject};
use poem::{
    handler,
    IntoResponse,
    web::{Data, Json, Path},
};
use sqlx::MySqlPool;

use crate::{
    database::entities::Guild,
    errors::{Error, GuildError},
};

#[handler]
pub async fn get_welcome_screen(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path(guild_id): Path<Snowflake>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    if !guild.has_member(db, claims.id).await? {
        return Err(Error::Guild(GuildError::MemberNotFound).into());
    }

    Ok(Json(guild.welcome_screen.0.to_owned()))
}

#[handler]
pub async fn modify_welcome_screen(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path(guild_id): Path<Snowflake>,
    Json(payload): Json<GuildModifyWelcomeScreenSchema>,
) -> poem::Result<impl IntoResponse> {
    let mut guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    if !guild.has_member(db, claims.id).await? {
        return Err(Error::Guild(GuildError::MemberNotFound).into());
    }

    if let Some(welcome_screen) = guild.welcome_screen.0.as_mut() {
        welcome_screen.description = payload.description;
        welcome_screen.enabled = payload.enabled.unwrap_or(welcome_screen.enabled);
        if let Some(welcome_channels) = payload.welcome_channels {
            welcome_screen.welcome_channels = welcome_channels;
        }
    } else {
        guild.welcome_screen = sqlx::types::Json(Some(WelcomeScreenObject {
            enabled: payload.enabled.unwrap_or_default(),
            description: payload.description,
            welcome_channels: payload.welcome_channels.unwrap_or(vec![]),
        }));
    }

    guild.save(db).await?;

    Ok(Json(guild.welcome_screen.0.to_owned()))
}
