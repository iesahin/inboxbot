use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    process::Command,
};

use chrono::Local;
use lazy_static::lazy_static;
use teloxide::{
    dispatching::{dialogue::InMemStorage, HandlerExt, MessageFilterExt},
    net::Download,
    prelude::*,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Inbox,
}

// pub enum State {
//     #[default]
//     Start,
//     ReceiveFullName,
//     ReceiveAge {
//         full_name: String,
//     },
//     ReceiveLocation {
//         full_name: String,
//         age: u8a
//     },
// }
//

// specify the username when compiling the binary
lazy_static! {
    static ref USERNAME: String = std::env::var("INBOXBOT_USERNAME").unwrap();
}

// const SAME_FILE_THRESHOLD: u64 = 1800;

// TODO: Add 'man and' to the exclude list
const EMOJI_EXCLUDE: Option<&str> = Some("flags");

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();
    let schema = Update::filter_message()
        .filter_map(|u: Update| u.from().cloned())
        .enter_dialogue::<Update, InMemStorage<State>, State>()
        .branch(Message::filter_document().endpoint(handle_document_message))
        .branch(Message::filter_photo().endpoint(handle_photo_message))
        .branch(Message::filter_audio().endpoint(handle_audio_message))
        .branch(Message::filter_voice().endpoint(handle_voice_message))
        .branch(Message::filter_text().endpoint(handle_text_message));

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn timestamp() -> String {
    Local::now().format("%Y%m%d%H%M%S").to_string()
}

async fn check_sender(bot: &Bot, msg: &Message) -> Result<bool, Box<dyn std::error::Error>> {
    let username = msg.from.as_ref().unwrap().username.clone();
    if username.unwrap() != USERNAME.to_owned() {
        bot.send_message(msg.chat.id, "You are not authorized to use this bot")
            .await?;
        return Ok(false);
    }
    Ok(true)
}

async fn handle_document_message(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Ok(false) = check_sender(&bot, &msg).await {
        return Ok(());
    }

    if let Some(document) = msg.document() {
        let document_file_id = &document.file.id;

        // Get the file information
        let file = bot.get_file(document_file_id).await?;

        // Download the file
        let file_path = file.path;
        let mut file_content = Vec::new();
        bot.download_file(&file_path, &mut file_content).await?;

        // Save the file to disk
        let file_name = format!("{}-{}", timestamp(), file_path.replace("/", "-"));
        fs::write(&file_name, &file_content)?;

        bot.send_message(msg.chat.id, format!("Document saved as {}", file_name))
            .await?;
    }

    Ok(())
}

async fn handle_voice_message(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Ok(false) = check_sender(&bot, &msg).await {
        return Ok(());
    }

    if let Some(voice) = msg.voice() {
        let voice_file_id = &voice.file.id;

        // Get the file information
        let file = bot.get_file(voice_file_id).await?;

        // Download the file
        let file_path = file.path;
        let mut file_content = Vec::new();
        bot.download_file(&file_path, &mut file_content).await?;

        // Save the file to disk
        let file_name = format!("{}-{}", timestamp(), file_path.replace("/", "-"));
        fs::write(&file_name, &file_content)?;

        bot.send_message(msg.chat.id, format!("Voice saved as {}", file_name))
            .await?;
    }

    Ok(())
}

async fn handle_audio_message(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Ok(false) = check_sender(&bot, &msg).await {
        return Ok(());
    }

    if let Some(audio) = msg.audio() {
        let audio_file_id = &audio.file.id;

        // Get the file information
        let file = bot.get_file(audio_file_id).await?;

        // Download the file
        let file_path = file.path;
        let mut file_content = Vec::new();
        bot.download_file(&file_path, &mut file_content).await?;

        // Save the file to disk
        let file_name = format!("{}-{}", timestamp(), file_path.replace("/", "-"));
        fs::write(&file_name, &file_content)?;

        bot.send_message(msg.chat.id, format!("Audio saved as {}", file_name))
            .await?;
    }

    Ok(())
}

async fn handle_photo_message(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Ok(false) = check_sender(&bot, &msg).await {
        return Ok(());
    }

    if let Some(photo) = msg.photo() {
        let largest_photo = photo.iter().max_by_key(|p| p.width * p.height).unwrap();
        let file_id = &largest_photo.file.id;

        // Get the file information
        let file = bot.get_file(file_id).await?;

        // Download the file
        let file_path = file.path;
        let mut file_content = Vec::new();
        bot.download_file(&file_path, &mut file_content).await?;

        // Save the file to disk
        let file_name = format!("{}-{}", timestamp(), file_path.replace("/", "-"));
        fs::write(&file_name, &file_content)?;

        bot.send_message(msg.chat.id, format!("Photo saved as {}", file_name))
            .await?;
    }

    Ok(())
}

async fn handle_text_message(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Ok(false) = check_sender(&bot, &msg).await {
        return Ok(());
    }

    if let Some(text) = msg.text() {
        // Check if the message is a command (starts with '/')
        if text.starts_with('/') {
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            let command = parts[0];

            match command {
                "/s" => {
                    if parts.len() > 1 {
                        let query = parts[1].trim();
                        match execute_search_command(query).await {
                            Ok(result) => {
                                // Send search results in chunks if they're too large
                                for chunk in result.chars().collect::<Vec<char>>().chunks(4000) {
                                    let chunk_text = chunk.iter().collect::<String>();
                                    bot.send_message(msg.chat.id, chunk_text).await?;
                                }
                            }
                            Err(e) => {
                                bot.send_message(
                                    msg.chat.id,
                                    format!("Error executing search: {}", e),
                                )
                                .await?;
                            }
                        }
                    } else {
                        bot.send_message(msg.chat.id, "Usage: /s <search_query>")
                            .await?;
                    }
                }
                _ => {
                    bot.send_message(msg.chat.id, format!("Unknown command: {}", command))
                        .await?;
                }
            }
        } else {
            // If not a command, process as a regular message
            let filename = write_message_to_file(msg.clone())?;
            append_to_file("\n", &filename)?;
            let words_in_file = count_words_in_file(&filename)?;
            bot.send_message(msg.chat.id, format!("{}", words_in_file))
                .await?;
            let random_emoji = randem::randem(None, None, EMOJI_EXCLUDE.map(|s| s.to_string()));
            bot.send_message(msg.chat.id, random_emoji.clone()).await?;
            // let time = Local::now().format("%H:%M").to_string();
            // append_to_file(&format!("@ {time} {random_emoji}\n"), &filename)?;
            dialogue.update(State::Inbox).await?;
        }
    }
    Ok(())
}

fn count_words_in_file(filename: &str) -> Result<usize, io::Error> {
    let content = fs::read_to_string(filename).unwrap_or_default();
    let word_count = content.split_whitespace().count();
    Ok(word_count)
}

fn append_to_file(text: &str, filename: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;

    file.write_all(text.as_bytes())?;
    Ok(())
}

async fn execute_search_command(query: &str) -> io::Result<String> {
    let output = Command::new("rg")
        .arg("--color=never")
        .arg("--line-number")
        .arg("--no-heading")
        .arg("--smart-case")
        .arg(query)
        .output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).into_owned();
        if result.is_empty() {
            Ok(format!("No matches found for query: '{}'", query))
        } else {
            Ok(result)
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr).into_owned();
        Ok(format!("Error executing search: {}", error))
    }
}

fn write_message_to_file(msg: Message) -> io::Result<String> {
    let mut text = if let Some(text) = msg.text() {
        text.to_owned()
    } else {
        String::new()
    };

    if let Some(entities) = msg.parse_entities() {
        for entity in entities {
            match entity.kind() {
                teloxide::types::MessageEntityKind::Url => {
                    let link = format!("[]({})\n", entity.text());
                    text.push_str(&link);
                }
                teloxide::types::MessageEntityKind::TextLink { url } => {
                    let link = format!("[{}]({})\n", entity.text(), url);
                    text.push_str(&link);
                }
                _ => {}
            }
        }
    }

    if let Some(forward) = msg.forward_from_user() {
        let date = if let Some(date) = msg.forward_date() {
            date.to_rfc2822()
        } else {
            "".to_owned()
        };

        let name = format!(
            "\nfrom: {} {} (at {})\n",
            forward.first_name,
            forward.last_name.as_deref().unwrap_or_default(),
            date,
        );
        text.push_str(&name);
    }

    let day = Local::now().format("%Y%m%d").to_string();
    let filename = format!("{}-tg.md", day);

    append_to_file(&text, &filename)?;
    Ok(filename)
}

// fn is_file_added_in_last_n_seconds(file_path: &str, n: u64) -> bool {
//     let re = Regex::new(r"^.*(d{14}).*md$").unwrap();
//
//     if let Some(captures) = re.captures(file_path) {
//         let timestamp_str = &captures[1];
//         if let Ok(parsed_timestamp) = DateTime::parse_from_str(timestamp_str, "%Y%m%d%H%M%S") {
//             let current_time = chrono::Utc::now().to_utc();
//             return parsed_timestamp
//                 > current_time
//                     .checked_sub_signed(TimeDelta::new(n as i64, 0).unwrap())
//                     .unwrap();
//         }
//     }
//     false
// }
