use core::str;

use config_file2::LoadConfigFile;
use die_exit::DieWith;
use futures_util::{
    stream::{self},
    StreamExt, TryStreamExt,
};
use log::{debug, error, info};
use sha3::{Digest, Sha3_256};
use teloxide::{net::Download, prelude::*};

use crate::{
    cli::Cli,
    config::{Config, CONFIG},
    db::{DbOperation, MarsImage, DB},
    utils::{config_path, msg_url, OnceLockDefaultInit},
};

async fn handler(bot: &Bot, message: Message) {
    // if `only_mars_for_channel_message` is set and the message is not sent by
    // channel
    if CONFIG.get_or_init_default().only_mars_for_channel_message && message.from().is_some() {
        return;
    }
    debug!(
        "get message from chat {}: id {}",
        message.chat.id, message.id
    );
    let (message_id, image_metas) = if let Some(image) = message.photo() {
        (message.id, image)
    } else {
        return;
    };
    let file_hash_stream = stream::iter(image_metas).filter_map(async |f| {
        let file_id = f.file.id.clone();
        let hash = match download_one_file_and_hash(bot, &file_id).await {
            Ok(x) => {
                if let Ok(hash_str) = str::from_utf8(&x) {
                    debug!("calc hash for file {file_id}: `{}`", hash_str);
                } else {
                    debug!("calc hash for file {file_id}: `{:?}`", x);
                };
                Some(x)
            }
            Err(err) => {
                error!("hashing file `{file_id}`: {err:?}");
                None
            }
        };
        hash.map(|hash| (file_id, hash))
    });
    file_hash_stream
        .for_each(async |(file_id, hash)| {
            let res = DB
                .insert_or_get_existing(
                    message.chat.id.0.to_string().as_str(),
                    &MarsImage::new(message_id.0, hash),
                )
                .die_with(|e| format!("Error while insert hash to database: {e:?}"));
            if let Some(id) = res {
                let origin_message_url =
                    msg_url(message.chat.id.0, id.id).expect("url concat number should be valid");
                let reply_text = format!("You Marsed! [Origin message]({origin_message_url})");
                bot.send_message(message.chat.id, reply_text)
                    .await
                    .log_on_error()
                    .await;
                info!(
                    "find mars message: {file_id} with {}, url: {}",
                    id.id, origin_message_url
                );
            }
        })
        .await;
}

async fn download_one_file_and_hash(
    bot: &Bot,
    file_id: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = bot.get_file(file_id).await?;
    let mut hasher = Sha3_256::new();
    while let Some(trunk) = bot.download_file_stream(&file.path).try_next().await? {
        hasher.update(trunk);
    }
    Ok(hasher.finalize().as_slice().to_vec())
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

    // Dispatcher::builder(bot, Update::filter_message().endpoint(handler))
    //     .default_handler(|_upd| Box::pin(async {}))
    //     .enable_ctrlc_handler()
    //     .build()
    //     .dispatch()
    //     .await;

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        handler(&bot, msg).await;
        Ok(())
    })
    .await;
}
