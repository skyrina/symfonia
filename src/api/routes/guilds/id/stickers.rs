use chorus::types::{
    GuildCreateStickerSchema, GuildModifyStickerSchema, jwt::Claims, Snowflake, StickerType,
};
use poem::{
    handler,
    http::StatusCode,
    IntoResponse,
    Response, web::{Data, Json, Multipart, Path},
};
use sqlx::MySqlPool;

use crate::{
    database::entities::{Guild, Sticker},
    errors::{Error, GuildError},
};

#[handler]
pub async fn get_stickers(
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

    let stickers = Sticker::get_by_guild(db, guild_id).await?;

    Ok(Json(stickers))
}

#[handler]
pub async fn create_sticker(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path(guild_id): Path<Snowflake>,
    sticker_data: Multipart,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    if !guild.has_member(db, claims.id).await? {
        return Err(Error::Guild(GuildError::MemberNotFound).into());
    }

    let sticker_data = GuildCreateStickerSchema::from_multipart(sticker_data).await?;

    let sticker = Sticker::create(
        db,
        Some(guild.id),
        None,
        Some(claims.id),
        &sticker_data.name,
        sticker_data.description,
        sticker_data.tags,
        StickerType::Guild,
        sticker_data.sticker_format_type,
    )
    .await?;

    // TODO: Emit event 'GUILD_STICKERS_UPDATE'

    Ok(Json(sticker))
}

#[handler]
pub async fn get_sticker(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path((guild_id, sticker_id)): Path<(Snowflake, Snowflake)>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    if !guild.has_member(db, claims.id).await? {
        return Err(Error::Guild(GuildError::MemberNotFound).into());
    }

    let sticker = Sticker::get_by_id(db, sticker_id)
        .await?
        .ok_or(Error::Guild(GuildError::StickerNotFound))?;

    if !sticker
        .guild_id
        .map(|gid| gid == guild.id)
        .unwrap_or_default()
    {
        return Err(Error::Guild(GuildError::StickerNotFound).into());
    }

    Ok(Json(sticker))
}

#[handler]
pub async fn modify_sticker(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path((guild_id, sticker_id)): Path<(Snowflake, Snowflake)>,
    Json(payload): Json<GuildModifyStickerSchema>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    if !guild.has_member(db, claims.id).await? {
        return Err(Error::Guild(GuildError::MemberNotFound).into());
    }

    let mut sticker = Sticker::get_by_id(db, sticker_id)
        .await?
        .ok_or(Error::Guild(GuildError::StickerNotFound))?;

    if !sticker
        .guild_id
        .map(|gid| gid == guild.id)
        .unwrap_or_default()
    {
        return Err(Error::Guild(GuildError::StickerNotFound).into());
    }

    if let Some(name) = payload.name {
        sticker.name = name;
    }
    sticker.description = payload.description;
    sticker.tags = payload.tags;

    sticker.save(db).await?;

    // TODO: Emit event 'GUILD_STICKERS_UPDATE'

    Ok(Json(sticker))
}

#[handler]
pub async fn delete(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path((guild_id, sticker_id)): Path<(Snowflake, Snowflake)>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    if !guild.has_member(db, claims.id).await? {
        return Err(Error::Guild(GuildError::MemberNotFound).into());
    }

    let sticker = Sticker::get_by_id(db, sticker_id)
        .await?
        .ok_or(Error::Guild(GuildError::StickerNotFound))?;

    if !sticker
        .guild_id
        .map(|gid| gid == guild.id)
        .unwrap_or_default()
    {
        return Err(Error::Guild(GuildError::StickerNotFound).into());
    }

    sticker.delete(db).await?;

    Ok(Response::builder().status(StatusCode::NO_CONTENT).finish())
}
