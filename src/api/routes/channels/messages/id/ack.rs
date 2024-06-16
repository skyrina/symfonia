use chorus::types::{jwt::Claims, MessageACK, Snowflake};
use poem::{
    handler,
    IntoResponse,
    web::{Data, Json, Path},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    database::entities::{Channel, ReadState},
    errors::{ChannelError, Error},
};

#[handler]
pub async fn acknowledge_message(
    Data(db): Data<&MySqlPool>,
    Data(claims): Data<&Claims>,
    Path(channel_id): Path<Snowflake>,
    Path(message_id): Path<Snowflake>,
    Json(payload): Json<MessageACK>,
) -> poem::Result<impl IntoResponse> {
    let channel = Channel::get_by_id(db, channel_id)
        .await?
        .ok_or(Error::Channel(ChannelError::InvalidChannel))?;

    // TODO: Check if user can view channel (VIEW_CHANNEL)

    if let Some(mut read_state) =
        ReadState::get_by_user_and_channel(db, channel_id, claims.id).await?
    {
        read_state.last_message_id = Some(message_id);
        read_state.save(db).await?;
    } else {
        ReadState::create(db, channel_id, claims.id, Some(message_id)).await?;
    }

    // TODO: emit events
    Ok(Json(json!({"token": null})))
}
