use std::{
    fs::{self, OpenOptions},
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use emoji::Emoji;
use rand::{seq::IteratorRandom, SeedableRng};

use chrono::Local;
use glob::glob;
use lazy_static::lazy_static;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

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
//         age: u8,
//     },
// }
//

// specify the username when compiling the binary
lazy_static! {
    static ref USERNAME: String = std::env::var("INBOXBOT_USERNAME").unwrap();
}

const SAME_FILE_THRESHOLD: u64 = 1800;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Inbox].endpoint(inbox)),
        // .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
        // .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
        // .branch(
        //     dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location),
        // )
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

fn transform_text(text: &str) -> String {
    // If the text starts with -, replace it with - [ ] to make it a markdown list item
    if let Some(text) = text.strip_prefix("- ") {
        format!("- [ ] {}", text)
    } else {
        text.to_string()
    }
}

async fn inbox(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = msg.from.as_ref().unwrap().username.clone();
    if username.unwrap() != USERNAME.to_owned() {
        bot.send_message(msg.chat.id, "You are not authorized to use this bot")
            .await?;
        return Ok(());
    }

    let mut filename: Option<String> = None;
    for entry in glob("*-tg.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if was_file_modified_in_last_n_seconds(&path.to_string_lossy(), SAME_FILE_THRESHOLD)
                {
                    filename = Some(path.to_string_lossy().to_string());
                    break;
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    filename = Some(write_message_to_file(msg.clone(), filename)?);
    let random_emoji = random_emoji(None);
    bot.send_message(msg.chat.id, random_emoji.clone()).await?;
    fs::write(
        PathBuf::from(&filename.unwrap()),
        format!("{random_emoji}\n"),
    )
    .unwrap();
    dialogue.update(State::Inbox).await?;
    Ok(())
}

fn write_message_to_file(msg: Message, path: Option<String>) -> io::Result<String> {
    let filename = match path {
        Some(p) => p,
        None => {
            let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
            format!("{}-tg.md", timestamp)
        }
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename.clone())?;

    if let Some(entities) = msg.parse_entities() {
        for entity in entities {
            match entity.kind() {
                teloxide::types::MessageEntityKind::Url => {
                    let link = format!("[]({})\n", entity.text());
                    file.write_all(link.as_bytes())?;
                }
                teloxide::types::MessageEntityKind::TextLink { url } => {
                    let link = format!("[{}]({})\n", entity.text(), url);
                    file.write_all(link.as_bytes())?;
                }
                _ => {}
            }
        }
    }

    msg.text()
        .map(|t| file.write_all(transform_text(t).as_bytes()));
    file.write_all(b"\n").unwrap();
    Ok(filename)
}

fn was_file_modified_in_last_n_seconds(file_path: &str, n: u64) -> bool {
    let metadata = fs::metadata(file_path).unwrap();
    let modified_time = metadata
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    current_time - modified_time < n
}

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn random_emoji(seed: Option<&str>) -> String {
    let mut rng = if let Some(seed) = seed {
        let hashed = hash_string(seed);
        rand::rngs::StdRng::seed_from_u64(hashed)
    } else {
        rand::rngs::StdRng::from_entropy()
    };

    let all_emoji: Vec<&Emoji> = emoji::lookup_by_name::iter_emoji().collect();
    let random_emoji = all_emoji.into_iter().choose(&mut rng).unwrap();
    random_emoji.glyph.to_owned()
}
