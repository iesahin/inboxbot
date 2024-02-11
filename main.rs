use std::{
    fs::{self, File, OpenOptions},
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

    if let Some(text) = msg.text() {
        // if there is a file changed within the last 60 seconds, append the text to it
        let mut found = false;
        for entry in glob("*-tg.md").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if was_file_modified_in_last_60_seconds(&path.to_string_lossy()).unwrap() {
                        let mut file = OpenOptions::new().append(true).open(path).unwrap();
                        file.write_all(text.as_bytes()).unwrap();
                        file.write_all(b"\n").unwrap();
                        found = true;
                        break;
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
        if !found {
            write_text_to_timestamped_file(text)?;
        }
        bot.send_message(msg.chat.id, random_emoji()).await?;
    }
    dialogue.update(State::Inbox).await?;
    Ok(())
}

// async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
//     match msg.text() {
//         Some(text) => {
//             bot.send_message(msg.chat.id, "How old are you?").await?;
//             dialogue
//                 .update(State::ReceiveAge {
//                     full_name: text.into(),
//                 })
//                 .await?;
//         }
//         None => {
//             bot.send_message(msg.chat.id, "Send me plain text.").await?;
//         }
//     }
//
//     Ok(())
// }

// async fn receive_age(
//     bot: Bot,
//     dialogue: MyDialogue,
//     full_name: String, // Available from `State::ReceiveAge`.
//     msg: Message,
// ) -> HandlerResult {
//     match msg.text().map(|text| text.parse::<u8>()) {
//         Some(Ok(age)) => {
//             bot.send_message(msg.chat.id, "What's your location?")
//                 .await?;
//             dialogue
//                 .update(State::ReceiveLocation { full_name, age })
//                 .await?;
//         }
//         _ => {
//             bot.send_message(msg.chat.id, "Send me a number.").await?;
//         }
//     }
//
//     Ok(())
// }
//
// async fn receive_location(
//     bot: Bot,
//     dialogue: MyDialogue,
//     (full_name, age): (String, u8), // Available from `State::ReceiveLocation`.
//     msg: Message,
// ) -> HandlerResult {
//     match msg.text() {
//         Some(location) => {
//             let report = format!("Full name: {full_name}\nAge: {age}\nLocation: {location}");
//             bot.send_message(msg.chat.id, report).await?;
//             dialogue.exit().await?;
//         }
//         None => {
//             bot.send_message(msg.chat.id, "Send me plain text.").await?;
//         }
//     }
//
//     Ok(())
// }

fn write_text_to_timestamped_file(text: &str) -> io::Result<()> {
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
    let filename = format!("{}-tg.md", timestamp);
    let mut file = File::create(filename)?;
    file.write_all(text.as_bytes())?;
    file.write_all(b"\n").unwrap();
    Ok(())
}

fn was_file_modified_in_last_60_seconds(file_path: &str) -> io::Result<bool> {
    let metadata = fs::metadata(file_path)?;
    let modified_time = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(current_time - modified_time < 60)
}

fn random_emoji() -> &'static str {
    let emojis = [
        "🌍", "🌎", "🌏", "🌐", "🗺️", "🗾", "🧭", "🏔️", "⛰️", "🌋", "🗻", "🏕️", "🏖️", "🏜️", "🏝️", "🏞️",
        "🏟️", "🏛️", "🏗️", "🧱", "🪨", "🪵", "🛖", "🏘️", "🏚️", "🏠", "🏡", "🏢", "🏣", "🏤", "🏥", "🏦",
        "🏨", "🏩", "🏪", "🏫", "🏬", "🏭", "🏯", "🏰", "💒", "🗼", "🗽", "⛪", "🕌", "🛕", "🕍",
        "⛩️", "🕋", "⛲", "⛺", "🌁", "🌃", "🏙️", "🌄", "🌅", "🌆", "🌇", "🌉", "♨️", "🎠", "🎡",
        "🎢", "💈", "🎪", "🚂", "🚃", "🚄", "🚅", "🚆", "🚇", "🚈", "🚉", "🚊", "🚝", "🚞", "🚋",
        "🚌", "🚍", "🚎", "🚐", "🚑", "🚒", "🚓", "🚔", "🚕", "🚖", "🚗", "🚘", "🚙", "🛻", "🚚",
        "🚛", "🚜", "🏎️", "🏍️", "🛵", "🦽", "🦼", "🛺",
    ];

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..emojis.len());
    emojis[index]
}
