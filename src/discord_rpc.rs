/* discord_rpc.rs
 *
 * SPDX-FileCopyrightText: © 2024 Brage Fuglseth
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::session_enums::*;
use discord_presence::models::rich_presence::Activity;
use discord_presence::Client;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::SystemTime;

const DISCORD_CLIENT_ID: u64 = 1320106636743802923;

enum RpcMessage {
    SendStored,
    Change(SessionType, SessionDuration, PresenceState),
    UpdateStats(f64, f64),
}

pub struct RpcWrapper {
    sender: Sender<RpcMessage>,
}

impl Default for RpcWrapper {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();

        let sender_clone = sender.clone();

        thread::spawn(move || {
            let start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_millis() as u64);

            let mut stored_activity = Activity::new();
            let mut stored_stats: Option<(f64, f64)> = None; // WPM & accuracy

            let mut client = Client::new(DISCORD_CLIENT_ID);

            client
                .on_connected(move |_ctx| {
                    sender_clone
                        .send(RpcMessage::SendStored)
                        .expect("channel exists until app shuts down");
                })
                .persist();

            client.start();

            for msg in receiver.iter() {
                if let RpcMessage::Change(session_type, duration, state) = msg {
                    let details_string = match session_type {
                        SessionType::Simple => format!("Simple, {}", duration.english_string()),
                        SessionType::Advanced => format!("Advanced, {}", duration.english_string()),
                        SessionType::Custom => "Custom text".to_string(),
                    };

                    stored_activity = Activity::new()
                        .state(state.english_string())
                        .details(details_string);
                } else if let RpcMessage::UpdateStats(wpm, accuracy) = msg {
                    stored_stats = Some((wpm, accuracy));
                }

                if let Some(time) = start_time {
                    stored_activity = stored_activity.timestamps(|t| t.start(time));
                }

                if let Some((wpm, accuracy)) = stored_stats {
                    let display_accuracy = (accuracy * 100.).floor() as usize;

                    stored_activity = stored_activity.assets(|a| {
                        a.large_image("main")
                            .large_text("Keypunch")
                            .small_image(wpm_to_image(wpm))
                            .small_text(format!(
                                "{:.0} WPM, {:.0}% correctness",
                                wpm.floor(),
                                display_accuracy
                            ))
                    });
                } else {
                    stored_activity =
                        stored_activity.assets(|a| a.large_image("main").large_text("Keypunch"));
                };

                let _ = client.set_activity(|mut a| {
                    a.clone_from(&stored_activity);
                    a
                });
            }
        });

        RpcWrapper { sender }
    }
}

impl RpcWrapper {
    pub fn set_activity(
        &self,
        session_type: SessionType,
        duration: SessionDuration,
        state: PresenceState,
    ) {
        self.sender
            .send(RpcMessage::Change(session_type, duration, state))
            .expect("channel exists until app shuts down");
    }

    pub fn set_stats(&self, wpm: f64, accuracy: f64) {
        self.sender
            .send(RpcMessage::UpdateStats(wpm, accuracy))
            .expect("channel exists until app shuts down");
    }
}

fn wpm_to_image(wpm: f64) -> &'static str {
    const WPM_IMAGES: &'static [(f64, &'static str)] = &[
        (10., "0-wpm"),
        (20., "10-wpm"),
        (30., "20-wpm"),
        (40., "30-wpm"),
        (50., "40-wpm"),
        (60., "50-wpm"),
        (70., "60-wpm"),
        (80., "70-wpm"),
        (90., "80-wpm"),
        (100., "90-wpm"),
        (110., "100-wpm"),
        (120., "110-wpm"),
        (130., "120-wpm"),
        (140., "130-wpm"),
        (150., "140-wpm"),
        (160., "150-wpm"),
        (170., "160-wpm"),
        (180., "170-wpm"),
        (190., "180-wpm"),
        (200., "190-wpm"),
    ];

    for (threshold, image) in WPM_IMAGES {
        if wpm < *threshold {
            return image;
        }
    }

    "200-wpm"
}
