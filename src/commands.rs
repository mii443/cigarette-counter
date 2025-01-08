use crate::database::DailySmokingSummary;
use crate::{Context, Error};
use chrono::Local;
use poise::serenity_prelude::{self as serenity, CreateInteractionResponseMessage};
use poise::CreateReply;

/// Creates a vector of buttons for each cigarette type.
///
/// # Arguments
/// * `ctx` - The context.
/// * `uuid` - A unique identifier for the interaction.
///
/// # Returns
/// A Result containing a vector of `serenity::CreateButton` or an `Error`.
async fn create_cigarette_buttons(
    ctx: &Context<'_>,
    uuid: &str,
) -> Result<Vec<serenity::CreateButton>, Error> {
    let db = ctx.data().database.lock().await;
    let cigarette_types = db.get_smoking_types().await?;

    Ok(cigarette_types
        .into_iter()
        .map(|cigarette_type| {
            serenity::CreateButton::new(format!("{}{}", uuid, cigarette_type.id))
                .style(serenity::ButtonStyle::Primary)
                .label(cigarette_type.description.unwrap_or_default())
        })
        .collect())
}

/// Formats the daily smoking summary into a string.
///
/// # Arguments
/// * `daily_summary` - A vector of `DailySmokingSummary`.
///
/// # Returns
/// A formatted string representing the daily smoking summary.
fn format_daily_summary(daily_summary: Vec<DailySmokingSummary>) -> String {
    daily_summary
        .into_iter()
        .map(|summary| {
            format!(
                "\n{}: {}本",
                summary.description,
                summary.total_quantity.unwrap_or_default()
            )
        })
        .collect()
}

/// Handles a component interaction.
///
/// # Arguments
/// * `ctx` - The context.
/// * `mci` - The component interaction.
/// * `uuid` - A unique identifier for the interaction.
///
/// # Returns
/// A Result indicating success or an `Error`.
async fn handle_interaction(
    ctx: &Context<'_>,
    mci: &serenity::ComponentInteraction,
    uuid: &str,
) -> Result<(), Error> {
    let db = ctx.data().database.lock().await;
    let user_id = mci.user.id.get().to_string();
    let user = db.get_or_create_user(&user_id, &ctx.author().name).await?;

    let cigarette_id = extract_cigarette_id(&mci.data.custom_id, uuid)?;

    db.log_smoking(&user.discord_id, cigarette_id, 1).await?;

    let daily_summary = db
        .get_daily_summary(&user.discord_id, Local::now().date_naive())
        .await?;

    let reply_content = format!(
        "記録しました。\n本日の累計本数{}",
        format_daily_summary(daily_summary)
    );

    mci.create_response(
        ctx,
        serenity::CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content(reply_content),
        ),
    )
    .await?;

    Ok(())
}

/// Extracts the cigarette ID from the custom ID.
///
/// # Arguments
/// * `custom_id` - The custom ID string.
/// * `uuid` - The unique identifier prefix.
///
/// # Returns
/// A Result containing the cigarette ID as an `i32` or an `Error`.
fn extract_cigarette_id(custom_id: &str, uuid: &str) -> Result<i32, Error> {
    i32::from_str_radix(custom_id.trim_start_matches(uuid), 10)
        .map_err(|e| Error::from(format!("Failed to parse cigarette ID: {}", e)))
}

/// Creates the cigarette counting user interface.
///
/// # Arguments
/// * `ctx` - The context.
///
/// # Returns
/// A Result indicating success or an `Error`.
#[poise::command(prefix_command)]
pub async fn create_cigarette_ui(ctx: Context<'_>) -> Result<(), Error> {
    let uuid = ctx.id().to_string();

    let buttons = create_cigarette_buttons(&ctx, &uuid).await?;
    let components = vec![serenity::CreateActionRow::Buttons(buttons)];
    let reply = CreateReply::default()
        .content("喫煙カウント")
        .components(components);

    ctx.send(reply).await?;

    while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx)
        .channel_id(ctx.channel_id())
        .filter({
            let uuid = uuid.clone();
            move |mci| mci.data.custom_id.starts_with(&uuid)
        })
        .await
    {
        handle_interaction(&ctx, &mci, &uuid).await?;
    }

    Ok(())
}
