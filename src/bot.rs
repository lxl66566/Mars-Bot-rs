use std::ops::Deref;

use config_file2::LoadConfigFile;
use die_exit::DieWith;
use futures_util::{
    io::BufWriter,
    stream::{self, FuturesOrdered, FuturesUnordered},
    StreamExt, TryStreamExt,
};
use log::{error, warn};
use sha3::{Digest, Sha3_256};
use teloxide::{dispatching::HandlerExt, net::Download, prelude::*, types::UpdateKind};

use crate::{
    config::Config,
    db::{DbBackend, DbOperation},
    utils::{config_path, msg_url},
};

#[derive(Debug)]
pub struct MarsBot {
    bot: Bot,
    db: DbBackend,
    config: Config,
}

impl Default for MarsBot {
    fn default() -> Self {
        MarsBot {
            bot: Bot::from_env(),
            db: DbBackend::new_binary().die_with(|e| format!("Cannot attach db backend: {e:?}")),
            config: Config::load_or_default(config_path())
                .die_with(|e| format!("Cannot read config from path `{:?}`: {e:?}", config_path())),
        }
    }
}

impl Deref for MarsBot {
    type Target = Bot;
    fn deref(&self) -> &Self::Target {
        &self.bot
    }
}

impl MarsBot {
    async fn handler(&self, message: Message) {
        let (message_id, image_metas) = if let Some(image) = message.photo() {
            (message.id, image)
        } else {
            return;
        };
        let file_hash_stream = stream::iter(image_metas).filter_map(async |f| {
            let file_id = f.file.id.clone();
            match self.download_one_file_and_hash(&file_id).await {
                Ok(x) => Some(x),
                Err(err) => {
                    error!("hashing file `{file_id}`: {err:?}");
                    None
                }
            }
        });
        file_hash_stream
            .for_each(async |hash| {
                let res = self
                    .db
                    .insert_or_get_existing(
                        message.chat.id.0.to_string().as_str(),
                        message_id.0,
                        &hash,
                    )
                    .die_with(|e| format!("Error while insert hash to database: {e:?}"));
                if let Some(id) = res {
                    let reply_text = format!(
                        "You Marsed! [Origin message]({})",
                        msg_url(message.chat.id.0, id.id)
                            .expect("url concat number should be valid")
                    );
                    self.bot
                        .send_message(message.chat.id, reply_text)
                        .await
                        .log_on_error()
                        .await;
                }
            })
            .await;
    }

    async fn download_one_file_and_hash(
        &self,
        file_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let file = self.bot.get_file(file_id).await?;
        let mut hasher = Sha3_256::new();
        while let Some(trunk) = self.bot.download_file_stream(&file.path).try_next().await? {
            hasher.update(trunk);
        }
        Ok(hasher.finalize().as_slice().to_vec())
    }

    async fn run(&self) {
        Dispatcher::builder(self.bot, Update::filter_message())
            .default_handler(async move |update| {
                if let UpdateKind::Message(message) = update.kind.to_owned() {
                    self.handler(message);
                }
            })
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}
