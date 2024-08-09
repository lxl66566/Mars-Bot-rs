use core::str;

use config_file2::LoadConfigFile;
use die_exit::DieWith;
use dyn_fmt::AsStrFormatExt;
use futures_util::{
    stream::{self},
    StreamExt, TryStreamExt,
};
use log::{debug, error, info, trace, warn};
use sha3::{Digest, Sha3_256};
use teloxide::{net::Download, prelude::*, types::ParseMode};

use crate::{
    cli::Cli,
    config::{Config, CONFIG},
    db::{MarsImage, DB},
    utils::{config_path, msg_url, OnceLockDefaultInit},
};

async fn handler(bot: &'static Bot, message: Message) {
    // if `only_mars_for_channel_message` is set and the message is not sent by
    // channel
    if CONFIG.get_or_init_default().only_mars_for_channel_message && message.from().is_some() {
        trace!("ignore message from user, because `only_mars_for_channel_message` is set");
        return;
    }
    if message.chat.is_private() {
        debug!("received message from private chat. It may not display a correct Mars link, because: https://t.me/withabsolutex/1841");
    }
    debug!(
        "get message from chat {}: id {}",
        message.chat.id, message.id
    );
    let (message_id, image_metas) = if let Some(image) = message.photo() {
        debug!("{} is a photo message", message.id);
        (message.id, image)
    } else {
        trace!("{} is not a photo message", message.id);
        return;
    };
    let owned_chat_id = message.chat.id.0.to_string();
    let chat_id = owned_chat_id.as_str();
    let chat_link = message.chat.invite_link();

    // get all file hash
    let file_hash_stream = stream::iter(image_metas)
        .map(async move |f: &teloxide::types::PhotoSize| {
            let file_id = f.file.id.clone();
            debug!(
                "file_id: {}, size: {}, Resolution: {}x{}",
                file_id, f.file.size, f.width, f.height
            );

            match download_one_file_and_hash(bot, &file_id).await {
                Ok(Some(x)) => {
                    debug!("calculate hash for file {file_id}: `{}`", hex::encode(&x));
                    Some(x)
                }
                Err(err) => {
                    error!("hashing file `{file_id}`: {err:?}");
                    None
                }
                Ok(None) => {
                    warn!("file `{file_id}` exceed size limit, do not record");
                    None
                }
            }
            .map(|hash| (file_id, hash))
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // get all conflict images
    let mut conflict = stream::iter(file_hash_stream)
        .filter_map(|(file_id, hash)| async move {
            let res = DB.insert_or_get_existing(chat_id, MarsImage::new(message_id.0, hash));
            match res {
                Ok(res) => Some((file_id, res)),
                Err(e) => {
                    error!("Error while insert hash to database: {e:?}");
                    None
                }
            }
        })
        .collect::<Vec<_>>()
        .await;

    // Only use one conflict image: if one image is conflict, it will conflict for
    // all four scaled images.
    if let Some((file_id, Some(image))) = conflict.pop() {
        let origin_message_url = msg_url(chat_link, message.chat.id.0, image.id);
        info!("find mars file: {file_id}, url: {}", origin_message_url);
        let reply_text = CONFIG
            .get_or_init_default()
            .mars_prompt
            .format(&[origin_message_url]);
        // .escape_telegram_markdown_text()
        bot.send_message(message.chat.id, reply_text)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_to_message_id(message_id)
            .await
            .log_on_error()
            .await;
    }
}

/// download a file, returns `Some(hash)` if hash successfully, or `Some(None)`
/// if file size is too big.
async fn download_one_file_and_hash(
    bot: &Bot,
    file_id: &str,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    let file = bot.get_file(file_id).await?;
    if file.size > CONFIG.get_or_init_default().max_file_size {
        return Ok(None);
    }
    trace!("download_file_path: {}", file.path);
    let mut hasher = Sha3_256::new();

    // download all trunks parallelly. code from https://github.com/capslock/stable-diffusion-bot/blob/main/crates/stable-diffusion-bot/src/bot/helpers.rs
    let bytes = bot
        .download_file_stream(&file.path)
        .try_collect()
        .await
        .map(bytes::BytesMut::freeze)?;
    hasher.update(bytes);
    Ok(Some(hasher.finalize().as_slice().to_vec()))
}

pub async fn run(cli: Cli) {
    let config_path = cli.config.unwrap_or_else(config_path);
    CONFIG.get_or_init(|| {
        Config::load_or_default(&config_path)
            .die_with(|e| format!("Cannot read config from path `{config_path:?}`: {e:?}"))
    });
    let bot = cli
        .token
        .or_else(|| CONFIG.get_or_init_default().token.clone())
        .map_or_else(Bot::from_env, Bot::new);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        handler(Box::leak(Box::new(bot)), msg).await;
        Ok(())
    })
    .await;
}
