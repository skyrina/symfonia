use chorus::types::{
    GuildDiscoveryHealthScore, GuildDiscoveryRequirements, jwt::Claims, Snowflake,
};
use poem::{
    handler,
    IntoResponse,
    web::{Data, Json, Path},
};
use sqlx::MySqlPool;

use crate::{
    database::entities::{Config, Guild},
    errors::{Error, GuildError},
};

#[handler]
pub async fn discovery_requirements(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Data(config): Data<&Config>,
    Path(guild_id): Path<Snowflake>,
) -> poem::Result<impl IntoResponse> {
    let guild = Guild::get_by_id(db, guild_id)
        .await?
        .ok_or(Error::Guild(GuildError::InvalidGuild))?;

    let mut discovery_requirements = GuildDiscoveryRequirements {
        guild_id: Some(guild.id),
        safe_environment: Some(true),
        healthy: Some(true),
        health_score_pending: Some(false),
        size: Some(true),
        nsfw_properties: None,
        protected: Some(true),
        sufficient: Some(true),
        sufficient_without_grace_period: Some(true),
        valid_rules_channel: Some(true),
        retention_healthy: Some(true),
        engagement_healthy: Some(true),
        age: Some(true),
        minimum_age: Some(0),
        health_score: Some(GuildDiscoveryHealthScore {
            avg_nonnew_communicators: 0,
            avg_nonnew_participators: 0,
            num_intentful_joiners: 0,
            perc_ret_w1_intentful: 0.0,
        }),
        minimum_size: Some(0),
        grace_period_end_date: None,
    };

    Ok(Json(discovery_requirements))
}
