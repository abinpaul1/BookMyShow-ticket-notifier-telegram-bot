use teloxide::RequestError;
use rusqlite::Result as SqlResult;

use crate::telegram_helper::TgBot;
use crate::bms_helper::BmsHelper;
use crate::db_helper::DbHelper;
use crate::utils::{Event, match_percent, is_date_within_2_weeks, is_date_in_past, is_valid_date_string};

use log::{info,warn,error};

// Controller handles interactions with BMS API and also sends messgae to TgBot to send to user
// All stateful data is fetched and stored from db using DbHelper

// IMPORTANT : Only Send back Telegram messages from functions that are solely triggered by user input

// Max enrollments per user is 3
const MAX_ALLOWED_ENROLLMENTS: usize = 3;

pub struct Controller{
    tg_bot: TgBot,
    bms_helper: BmsHelper,
    db_helper: DbHelper
}

impl Controller{
    pub fn new(bot: TgBot) -> Controller{
        Controller{
            tg_bot: bot,
            bms_helper: BmsHelper::new(),
            db_helper: DbHelper::new()
        }
    }

    // Enroll a chat id into a list of hashmap of a given (movie,venue,date)
    pub async fn enroll_user(&mut self, chat_id: i64, movie_code: String, venue_code: String, date_str: String){
        info!("enroll_user: chat_id: {}, movie_code: {}, venue_code: {}, date: {}", chat_id, movie_code, venue_code, date_str);

        // Check if user has already exceeded MAX_ALLOWED_ENROLLMENTS
        let num_enrollments = match self.db_helper.get_number_enrollments(chat_id){
            Ok(num_enrollments) => num_enrollments,
            Err(e) => {
                error!("Error in fetching number of enrollments from db {}", e);
                self.tg_bot.send_message(chat_id, "Error : Internal error. Please try again").await.unwrap();
                return;
            }
        };
        if num_enrollments >= MAX_ALLOWED_ENROLLMENTS {
            self.tg_bot.send_limited_enrollments_message(chat_id).await.unwrap();
            return;
        }
        
        // Validate date and timeframe
        if !is_valid_date_string(&date_str) {
            self.tg_bot.send_message(chat_id, "Error : Provided date format is invalid").await.unwrap();
            return;
        }
        if !is_date_within_2_weeks(&date_str){
            self.tg_bot.send_message(chat_id, "Error : Date is not within next 2 weeks").await.unwrap();
            return;
        }

        // Get the movie name from BMS api
        let movie_name = match self.bms_helper.api_get_movie_name_from_code(&movie_code).await.unwrap(){
            Some(movie_name) => movie_name,
            None => {
                self.tg_bot.send_message(chat_id, "Error : Movie code is wrong").await.unwrap();
                return
            },
        };
        

        // Validating venue code
        let venue_name = match self.bms_helper.api_get_venue_name_from_code(&venue_code).await.unwrap(){
            Some(venue_name) => venue_name,
            None => {
                self.tg_bot.send_message(chat_id, "Error : Venue code is wrong").await.unwrap();
                return
            },
        };

        // Storing the venue name in db
        match self.db_helper.insert_venue(&venue_code, &venue_name){
            Ok(_) => {},
            Err(e) => {
                error!("Error while inserting venue into DB : {:?}", e);
                self.tg_bot.send_message(chat_id, "Error : Internal error. Please try again").await.unwrap();
                return
            }
        };

        // Create the event
        let event = Event::new(movie_name.clone(), venue_code.clone(), date_str.clone());
        match self.add_user_to_waitlist(chat_id, event){
            Ok(_) => {},
            Err(e) => {
                error!("Error while adding user to waitlist : {:?}", e);
                self.tg_bot.send_message(chat_id, "Error : Internal error. Please try again").await.unwrap();
                return;
            }
        };

        // Notify user that he was enrolled successfully
        match self.tg_bot.notify_enrollment_success(chat_id, &movie_name, &venue_name, &date_str).await{
            Ok(_) => {},
            Err(e) => {
                error!("Error while notifying enrollment success : {:?}", e);
                dbg!(e);
            }
        };
    }

    // Notify all users that the given event has started booking and remove them from waiting list
    async fn booking_started(&mut self, event: &Event) -> Result<(), RequestError>{
        info!("booking_started: For {:?}", event);

        // Notify all chat_id corresponding to event
        let waiting_list_for_event = self.db_helper.get_waiting_users(event).unwrap();
        info!("booking_started: Notifying {} users for {:?}", waiting_list_for_event.len(), event);
        for chat_id in waiting_list_for_event.into_iter(){
            // Send message to each chat_id using tg_bot
            let venue_name = self.db_helper.get_venue_name(&event.venue_code).unwrap();
            self.tg_bot.notify_booking_started(chat_id, &event.movie_name, &venue_name, &event.date_string).await?;
        }

        // Remove the events and users from db
        match self.db_helper.remove_event(event){
            Ok(_) => {},
            Err(e) => {
                error!("Error removing event from db: {:?}", e);
            }
        }
        Ok(())
    }

    pub async fn list_locations(&self, chat_id: i64) -> Result<(), RequestError> {
        info!("list_locations");
        let locations = self.bms_helper.api_get_all_locations().await.unwrap();
        self.tg_bot.send_locations(chat_id, locations).await
    }

    pub async fn list_venues_for_location(&self, chat_id: i64, location_code: &str) -> Result<(), RequestError> {
        info!("list_venues_for_location: location_code: {}", location_code);
        let venues = self.bms_helper.api_get_venues_for_region(location_code).await.unwrap();
        self.tg_bot.send_available_venues_at_location(chat_id, location_code, venues).await
    }

    pub async fn send_help_message(&self, chat_id: i64){
        info!("Sending help message to {}", chat_id);
        match self.tg_bot.send_help_message(chat_id).await{
            Ok(()) => {},
            Err(e) => {
                error!("Error in sending message using TgBot: {}", e);
                dbg!(e);
            }
        }
    }

    // Add user to waiting list, if the event is not in the waiting list, adds it
    fn add_user_to_waitlist(&mut self, chat_id: i64, event: Event) -> SqlResult<()>{
        info!("add_user_to_waitlist: chat_id: {}, event: {:?}", chat_id, event);
        // Add user to db and waiting list
        self.db_helper.insert_user(chat_id, &event)?;
        Ok(())
    }

    pub async fn get_waiting_list_for_user(&self, chat_id: i64) -> Result<(), RequestError>{
        info!("get_waiting_list_for_user: chat_id: {}", chat_id);

        let wl = self.db_helper.get_events_for_user(chat_id).unwrap();
        let waiting_list = wl.iter().map(|event| {
            let venue_name = match self.db_helper.get_venue_name(&event.venue_code){
                // If venue name not found, we use venue code as venue name
                Ok(venue_name) => venue_name,
                Err(_) => {
                    info!("Venue name not found for venue {:?}", event.venue_code);    
                    event.venue_code.clone()
                }
            };
            (event.movie_name.clone(), venue_name, event.date_string.clone())
        }).collect::<Vec<(String, String, String)>>();

        self.tg_bot.send_waiting_list_for_user(chat_id, waiting_list).await
    }

    // Does a check if the events in waiting list have shows now
    pub async fn poll_bms(&mut self){
        warn!("poll_bms called at {}", chrono::Local::now());
        let waiting_list = match self.db_helper.get_all_events_and_users(){
            Ok(wl) => wl,
            Err(e) => {
                error!("Error while getting waiting list from db: {:?}", e);
                return
            }
        };
        // For each event in waiting list
        // Make request using venue code and date
        // If movie is present in response, notify all users in waiting list
        for event in waiting_list.keys(){
            let movie_name = &event.movie_name;
            let venue_code = &event.venue_code;
            let date_string = &event.date_string;

            // If date is past, remove from waiting list
            if is_date_in_past(date_string){
                info!("poll_bms: Event is in past, removing from waiting list: {:?}", event);
                match self.db_helper.remove_event(event){
                    Ok(_) => (),
                    Err(e) => error!("poll_bms: Error removing event from DB: {:?}", e),
                };
                continue;
            }


            let available_movies = match self.bms_helper.api_get_movies_at_venue(venue_code, date_string).await{
                Ok(movies) => movies,
                Err(e) => {
                    error!("poll_bms: Error getting movies at venue : {:?} ", e);
                    continue;
                }
            };
            
            // If returned list contains movie name, notify all users in waiting list
            // Fuzzy matching :  > 0.75 percent match on name
            for available_movie in available_movies.iter(){
                if match_percent(movie_name, available_movie) > 0.75{
                    info!("Matched {} from waiting list with {}", movie_name, available_movie);
                    self.booking_started(event).await.unwrap();
                    break; 
                }
            }
        }
    }
}

