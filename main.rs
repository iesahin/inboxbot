use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::Rng;

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
const TIMESTAMP_FORMAT: &str = "%Y%m%d%H%M%S"
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

async fn inbox(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = msg.from().unwrap().username.clone();
    if username.unwrap() != USERNAME.to_owned() {
        bot.send_message(msg.chat.id, "You are not authorized to use this bot")
            .await?;
        return Ok(());
    }

    let mut found = false;
    for entry in glob("*-tg.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if is_file_created_in_last_n_seconds(&path.to_string_lossy(), SAME_FILE_THRESHOLD)
                {
                    write_message_to_file(msg.clone(), Some(path.to_string_lossy().to_string()))?;
                    found = true;
                    break;
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    if !found {
        write_message_to_file(msg.clone(), None)?;
    }
    bot.send_message(msg.chat.id, random_emoji()).await?;
    dialogue.update(State::Inbox).await?;
    Ok(())
}

fn write_message_to_file(msg: Message, path: Option<String>) -> io::Result<()> {
    let filename = match path {
        Some(p) => p,
        None => {
            let timestamp = Local::now().format(TIMESTAMP_FORMAT).to_string();
            format!("{}-tg.md", timestamp)
        }
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;

    if let Some(entities) = msg.parse_entities() {
        for entity in entities {
            match entity.kind() {
                teloxide::types::MessageEntityKind::Url => {
                    let link = format!("- []({})\n", entity.text());
                    file.write_all(link.as_bytes())?;
                }
                teloxide::types::MessageEntityKind::TextLink { url } => {
                    let link = format!("- [{}]({})\n", entity.text(), url);
                    file.write_all(link.as_bytes())?;
                }
                _ => {}
            }
        }
    }

    msg.text().map(|t| file.write_all(t.as_bytes()));
    file.write_all(b"\n").unwrap();
    Ok(())
}

fn was_file_modified_in_last_n_seconds(file_path: &str, n: u64) -> bool {
    let metadata = fs::metadata(file_path).unwrap();
    let modified_time = metadata
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    current_time - modified_time < n
}

fn is_file_created_in_last_n_seconds(file_path: &str, 
n: u64) -> bool {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();


   if let Ok(creation_time) = parse_datetime_to_timestamp(file_path) {
       current_time - creation_time < n
   } else {
       false
    }
}   
   
  
use chrono::{NaiveDateTime, TimeZone, Utc};

fn parse_datetime_to_timestamp(datetime_str: &str) -> Result<i64, Error> {
    // Extract the datetime portion from the string
    let datetime_portion = &datetime_str[..14];

    // Parse the datetime string into a NaiveDateTime object
    let naive_datetime = NaiveDateTime::parse_from_str(datetime_portion, TIMESTAMP_FORMAT)?;

    // Convert NaiveDateTime to a timestamp integer (seconds since Unix epoch)
    let timestamp = Utc.from_utc_datetime(&naive_datetime).timestamp();

    Ok(timestamp)
}



fn random_emoji() -> &'static str {
    let emojis = [
        "🌍", "🌎", "🌏", "🌐", "🗺️", "🗾", "🧭", "🏔️", "⛰️", "🌋", "🗻", "🏕️", "🏖️", "🏜️", "🏝️", "🏞️",
        "🏟️", "🏛️", "🏗️", "🧱", "🪨", "🪵", "🛖", "🏘️", "🏚️", "🏠", "🏡", "🏢", "🏣", "🏤", "🏥", "🏦",
        "🏨", "🏩", "🏪", "🏫", "🏬", "🏭", "🏯", "🏰", "💒", "🗼", "🗽", "⛪", "🕌", "🛕", "🕍",
        "⛩️", "🕋", "⛲", "⛺", "🌁", "🌃", "🏙️", "🌄", "🌅", "🌆", "🌇", "🌉", "♨️", "🎠", "🎡",
        "🎢", "💈", "🎪", "🚂", "🚃", "🚄", "🚅", "🚆", "🚇", "🚈", "🚉", "🚊", "🚝", "🚞", "🚋",
        "🚌", "🚍", "🚎", "🚐", "🚑", "🚒", "🚓", "🚔", "🚕", "🚖", "🚗", "🚘", "🚙", "🛻", "🚚",
        "🚛", "🚜", "🏎️", "🏍️", "🛵", "🦽", "🦼", "🛺", "🐶", "🐱", "🐭", "🐹", "🐰", "🦊", "🐻",
        "🐼", "🐨", "🐯", "🦁", "🐮", "🐸", "🐵", "🐔", "🐧", "🐦", "🐤", "🦆", "🦅", "🦉", "🦇",
        "🐺", "🐗", "🐴", "🦄", "🐝", "🐛", "🦋", "🐌", "🐞", "🐜", "🦟", "🦗", "🕷", "🕸", "🐢",
        "🐍", "🦎", "🦂", "🦀", "🦞", "🦐", "🦑", "🐙", "🦕", "🦖", "🐳", "🐋", "🐬", "🐟", "🐠",
        "🐡", "🦈", "🐊", "🐅", "🐆", "🦓", "🦍", "🐘", "🦏", "🦛", "🐪", "🐫", "🦒", "🦘", "🦬",
        "🐃", "🐂", "🐄", "🐎", "🐏", "🐑", "🐐", "🦌", "🐕", "🐩", "🦮", "🐕", "🐈", "🐓", "🦃",
        "🦚", "🦜", "🦢", "🦩", "🕊", "🐇", "🌱", "🌲", "🌳", "🌴", "🌵", "🌾", "🌿", "☘️", "🍀",
        "🍁", "🍂", "🍃", "🪴", "🎋", "🎍", "🌺", "🌻", "🌼", "🌷", "🌹", "🥀", "🌸", "💐", "🍄",
        "🌰", "🎄", "🌼", "🌻", "🌞", "🌝",
    ];

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..emojis.len());
    emojis[index]
}
