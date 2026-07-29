use crate::config::RpcRule;
use crate::logger::Logger;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};

pub struct PresenceHandle {
    client: DiscordIpcClient,
}

impl PresenceHandle {
    pub fn new(id: &str) -> Self {
        Self {
            client: DiscordIpcClient::new(id),
        }
    }

    pub fn open(&mut self) -> Result<(), String> {
        self.client
            .connect()
            .map_err(|e| format!("Discord connection failed: {e}"))
    }

    fn make_activity(rule: &RpcRule) -> activity::Activity<'_> {
        let mut act = activity::Activity::new();

        if let Some(ref s) = rule.state {
            act = act.state(s.as_str());
        }

        if let Some(ref d) = rule.details {
            act = act.details(d.as_str());
        }

        if rule.large_image.is_some()
            || rule.small_image.is_some()
            || rule.large_text.is_some()
            || rule.small_text.is_some()
        {
            let mut assets = activity::Assets::new();

            if let Some(ref v) = rule.large_image {
                assets = assets.large_image(v.as_str());
            }
            if let Some(ref v) = rule.large_text {
                assets = assets.large_text(v.as_str());
            }
            if let Some(ref v) = rule.small_image {
                assets = assets.small_image(v.as_str());
            }
            if let Some(ref v) = rule.small_text {
                assets = assets.small_text(v.as_str());
            }

            act = act.assets(assets);
        }

        act
    }

    pub fn push(&mut self, rule: &RpcRule, label: &str) {
        let activity = Self::make_activity(rule);

        Logger::log(&format!(
            "[RPC] window={label} state={:?} details={:?} img_l={:?} txt_l={:?} img_s={:?} txt_s={:?}",
            rule.state,
            rule.details,
            rule.large_image,
            rule.large_text,
            rule.small_image,
            rule.small_text,
        ));

        if let Err(e) = self.client.set_activity(activity) {
            Logger::log(&format!("[RPC] set_activity failed ({e}), reconnecting..."));
            if let Err(r) = self.open() {
                Logger::log(&format!("[RPC] reconnect failed: {r}"));
                return;
            }
            let retry = Self::make_activity(rule);
            if let Err(e2) = self.client.set_activity(retry) {
                Logger::log(&format!("[RPC] retry also failed: {e2}"));
            }
        }
    }
}
