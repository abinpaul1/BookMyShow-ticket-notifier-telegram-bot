use teloxide::prelude::*;
use teloxide::dispatching::update_listeners::{
    polling_default, AsUpdateStream,
};
use teloxide::types::UpdateKind;
use tokio::time::{sleep, Duration};
use log::{info,warn,error};

mod notification_controller;
mod telegram_helper;
mod bms_helper;
mod db_helper;
mod utils;


#[tokio::main]
async fn main() {
    run().await;
}


// Utility funvtion to parse message and extract command and arguments as a tuple
fn parse_user_command(message: &str) -> Option<(&str, Vec<&str>)> {
    let mut split = message.split_whitespace();
    let command = split.next();
    let args = split.collect::<Vec<_>>();
    command.map(|command| (command, args))
}


async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting TellMyShow Bot...");

    let bot = Bot::from_env();

    // Controller for polling bms every 5 minutes using tokio spawn
    let tg_bot = telegram_helper::TgBot::new(bot.clone());
    let mut controller = notification_controller::Controller::new(tg_bot);

    let poller = async move {
        loop {
            sleep(Duration::from_secs(360)).await;
            controller.poll_bms().await;
        }
    };
    tokio::spawn(poller);



    // Controller for doing updates
    let tg_bot = telegram_helper::TgBot::new(bot.clone());
    let controller = notification_controller::Controller::new(tg_bot);


    
    // Handle all kind of messages with our handlers
    let updater = polling_default(bot.clone());
    updater.await.as_stream().fold(controller, |mut controller, update| async {
        match update{
            Ok(update) => match update.kind {
                UpdateKind::Message(message) => {
                    let chat_id = message.chat_id();
                    // Handle message
                    match message.text() {
                        Some(text) => {
                            // Handle message
                            let (command, args) = parse_user_command(text).unwrap();
                            match command {
                                "/list_locations" => {
                                    match controller.list_locations(chat_id).await{
                                        Ok(()) => {},
                                        Err(e) => {
                                            error!("Error in /list_locations: {}", e);
                                            dbg!(e);
                                        }
                                    }
                                },
                                "/list_venues" => {
                                    // Command is of the form "/list_venues <location_code>"
                                    if args.len() != 1 {
                                        // Send back error message and return
                                        warn!("/list_venues Invalid Arguments");
                                        controller.send_help_message(chat_id).await;
                                        return controller;
                                    }
                                    match controller.list_venues_for_location(chat_id, &args[0].to_uppercase()).await{
                                        Ok(()) => {},
                                        Err(e) => {
                                            error!("Error in /list_venues: {}", e);
                                            dbg!(e);
                                        }
                                    };
                                },
                                "/enroll" => {
                                    // Command is of the form "/enroll <movie_code> <venue_code> <date_string>"    
                                    if args.len() != 3 {
                                        // Send back error message and return
                                        warn!("/enroll Invalid Arguments");
                                        controller.send_help_message(chat_id).await;
                                        return controller;
                                    }
                                    info!("/enroll called with args : {:?}", args);
                                    controller.enroll_user(chat_id, args[0].to_string().to_uppercase(), args[1].to_uppercase(), args[2].to_string()).await;                         
                                },
                                "/wl" => {
                                    info!("/wl called for chat id : {}", chat_id);
                                    match controller.get_waiting_list_for_user(chat_id).await{
                                        Ok(()) => {},
                                        Err(e) => {
                                            error!("Error in /wl: {}", e);
                                            dbg!(e);
                                        }
                                    }                        
                                },
                                _ => {
                                    // Send help message of available commands
                                    info!("UNKNOWN MESSAGE : {}",command);
                                    controller.send_help_message(chat_id).await;
                                }
                            }
                        },
                        None => {
                            // Handle non-text message
                            warn!("Non-text message received");
                        }
                    }
                },
                UpdateKind::CallbackQuery(_callback_query) => {
                    // Handle callback query
                    warn!("UpdateKind::CallbackQuery received");
                },
                _ => {
                    // Handle other kinds of updates
                    warn!("UpdateKind::Other received");
                }
            },
            _ => {
                // Err handling
                error!("Error in update received");
            }
        }
        controller
    }).await;
}


