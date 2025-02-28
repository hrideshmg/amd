/*
amFOSS Daemon: A discord bot for the amFOSS Discord server.
Copyright (C) 2024 amFOSS

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use crate::{
    graphql::fetch_members,
    utils::{get_five_am_timestamp, time_until},
};
use async_trait::async_trait;
use serenity::{
    all::{ChannelId, Message},
    client::Context,
};

use tokio::time::Duration;

const GROUP_ONE_CHANNEL_ID: u64 = 1225098248293716008;
const GROUP_TWO_CHANNEL_ID: u64 = 1225098298935738489;
const GROUP_THREE_CHANNEL_ID: u64 = 1225098353378070710;
const GROUP_FOUR_CHANNEL_ID: u64 = 1225098407216156712;
const STATUS_UPDATE_CHANNEL_ID: u64 = 764575524127244318;

#[async_trait]
pub trait Task: Send + Sync {
    fn name(&self) -> &'static str;
    fn run_in(&self) -> Duration;
    async fn run(&self, ctx: Context);
}

pub struct StatusUpdateCheck;

#[async_trait]
impl Task for StatusUpdateCheck {
    fn name(&self) -> &'static str {
        "StatusUpdateCheck"
    }

    fn run_in(&self) -> Duration {
        time_until(5, 0)
    }

    async fn run(&self, ctx: Context) {
        let members = fetch_members().await.expect("Root must be up.");

        let channel_ids: Vec<ChannelId> = vec![
            ChannelId::new(GROUP_ONE_CHANNEL_ID),
            ChannelId::new(GROUP_TWO_CHANNEL_ID),
            ChannelId::new(GROUP_THREE_CHANNEL_ID),
            ChannelId::new(GROUP_FOUR_CHANNEL_ID),
        ];

        let time = chrono::Local::now().with_timezone(&chrono_tz::Asia::Kolkata);
        let today_five_am = get_five_am_timestamp(time);
        let yesterday_five_am = today_five_am - chrono::Duration::hours(24);

        let mut valid_updates: Vec<Message> = vec![];

        for &channel_id in &channel_ids {
            let builder = serenity::builder::GetMessages::new().limit(50);
            match channel_id.messages(&ctx.http, builder).await {
                Ok(messages) => {
                    let filtered_messages: Vec<Message> = messages
                        .into_iter()
                        .filter(|msg| {
                            let msg_content = msg.content.to_lowercase();
                            msg.timestamp >= yesterday_five_am.into()
                                && msg.timestamp < today_five_am.into()
                                && msg_content.contains("namah shivaya")
                                && msg_content.contains("regards")
                        })
                        .collect();

                    valid_updates.extend(filtered_messages);
                }
                Err(e) => println!("ERROR: {:?}", e),
            }
        }

        let mut naughty_list: Vec<String> = vec![];

        for member in &members {
            let name_parts: Vec<&str> = member.split_whitespace().collect();
            let first_name = name_parts.get(0).unwrap_or(&"");
            let last_name = name_parts.get(1).unwrap_or(&"");
            let has_sent_update = valid_updates
                .iter()
                .any(|msg| msg.content.contains(first_name) || msg.content.contains(last_name));

            if !has_sent_update {
                naughty_list.push(member.to_string());
            }
        }

        let status_update_channel = ChannelId::new(STATUS_UPDATE_CHANNEL_ID);

        if naughty_list.is_empty() {
            status_update_channel
                .say(ctx.http, "Everyone sent their update today!")
                .await;
        } else {
            let formatted_list = naughty_list
                .iter()
                .enumerate()
                .map(|(i, member)| format!("{}. {:?}", i + 1, member))
                .collect::<Vec<String>>()
                .join("\n");
            status_update_channel
                .say(
                    ctx.http,
                    format!(
                        "These members did not send their updates:\n{}",
                        formatted_list
                    ),
                )
                .await;
        }
    }
}

pub fn get_tasks() -> Vec<Box<dyn Task>> {
    vec![Box::new(StatusUpdateCheck)]
}
