use chorus::types::{GuildTemplateCreateSchema, jwt::Claims, Snowflake};
use poem::{
    handler,
    IntoResponse,
    web::{Data, Json, Path},
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    database::entities::{Config, Guild, GuildTemplate, User},
    errors::{Error, GuildError},
};

#[handler]
pub async fn get_template(
    Data(db): Data<&MySqlPool>,
    Data(config): Data<&Config>,
    Path(code): Path<String>,
) -> poem::Result<impl IntoResponse> {
    if !config.templates.enabled {
        return Ok(Json(json!({
            "code": 403,
            "message": "Template creation & usage is disabled on this instance."
        }))
        .with_status(StatusCode::UNAUTHORIZED)
        .into_response());
    }

    if code.starts_with("discord:") {
        if !config.templates.allow_discord_templates {
            return Ok(Json(json!({
                "code": 403,
                "message": "Discord templates cannot be used on this instance."
            }))
            .with_status(StatusCode::UNAUTHORIZED)
            .into_response());
        }

        let (_, discord_template_id) = code.split_once(':').unwrap();
        let discord_template_data: GuildTemplate = reqwest::get(&format!(
            "https://discord.com/api/v9/guilds/templates/{}",
            discord_template_id
        ))
        .await
        .map_err(Error::from)?
        .json()
        .await
        .map_err(Error::from)?;

        return Ok(Json(discord_template_data).into_response());
    }

    if code.starts_with("external:") {
        if !config.templates.allow_raws {
            return Ok(Json(json!({
                "code": 403,
                "message": "Importing raw templates is disabled on this instance."
            }))
            .with_status(StatusCode::UNAUTHORIZED)
            .into_response());
        }

        let (_, data) = code.split_once(':').unwrap();

        let template: GuildTemplate = serde_json::from_str(data).map_err(Error::from)?;

        return Ok(Json(template).into_response());
    }

    let template = GuildTemplate::get_by_code(db, &code)
        .await?
        .ok_or(Error::Guild(GuildError::TemplateNotFound))?;

    Ok(Json(template).into_response())
}

#[handler]
pub async fn create_guild_from_template(
    Data(db): Data<&MySqlPool>,
    Data(authed_user): Data<&User>,
    Data(config): Data<&Config>,
    Path(code): Path<String>,
    Json(payload): Json<GuildTemplateCreateSchema>,
) -> poem::Result<impl IntoResponse> {
    if !config.templates.enabled {
        return Ok(Json(json!({
            "code": 403,
            "message": "Template creation & usage is disabled on this instance."
        }))
        .with_status(StatusCode::UNAUTHORIZED)
        .into_response());
    }

    if !config.templates.enabled {
        return Ok(Json(json!({
            "code": 403,
            "message": "Template creation is disabled on this instance."
        }))
        .with_status(StatusCode::UNAUTHORIZED)
        .into_response());
    }

    let guild_count = authed_user.count_guilds(db).await?;

    if guild_count >= config.limits.user.max_guilds as i32 {
        unimplemented!("Fail out due to guild limit")
    }

    let template = GuildTemplate::get_by_code(db, &code)
        .await?
        .ok_or(Error::Guild(GuildError::TemplateNotFound))?;

    let guild =
        Guild::create_from_template(db, config, authed_user.id, &template, &payload.name).await?;

    guild.add_member(db, authed_user.id).await?;

    Ok(Json(json!({
        "id": guild.id,
    }))
    .with_status(StatusCode::CREATED)
    .into_response())
}
