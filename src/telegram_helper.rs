use teloxide::prelude::*;
use teloxide::RequestError;
use teloxide::types::ParseMode::Html;

// Tg Bot helper, takes data and formats it well and sends it to the user

pub struct TgBot<> {
    pub bot: Bot,
}

// TODO : Extract all response messages to TgResponse Module

impl TgBot {
    // new bot
    pub fn new(bot: Bot) -> TgBot {
        TgBot { bot }
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), RequestError> {
        let a = self.bot.send_message(chat_id, text)
            .parse_mode(Html)
            .disable_web_page_preview(true)
            .send()
            .await;
        a.map(|_| ())
    }

    // Notify user that a given movie, venue, date has started booking
    pub async fn notify_booking_started(&self, chat_id: i64, movie: &str, venue: &str, date: &str) -> Result<(), RequestError> {
        let text = format!("Booking for {} at {} on {} has started", movie, venue, date);
        self.send_message(chat_id, &text).await
    }

    // Notify successfull enrollment
    pub async fn notify_enrollment_success(&self, chat_id: i64, movie: &str, venue: &str, date: &str) -> Result<(), RequestError> {
        let text = format!("You will be notified via a message here when booking opens for {} at {} on {}", movie, venue, date);
        self.send_message(chat_id, &text).await
    }

    // Send list of available locations
    pub async fn send_locations(&self, chat_id: i64, locations: Vec<(String, String)>) -> Result<(), RequestError> {
        let mut text ="Available locations are: \n\n".to_string();
        for (i, loc) in locations.iter().enumerate(){
            let loc_text = format!("{}. {} - <pre>{}</pre>\n\n",i+1, loc.0, loc.1);
            text.push_str(&loc_text.to_string());

            // If length of text exceed 3500 (limit is 4096), send it and start a new one
            if text.len() > 3500 {
                self.send_message(chat_id, &text).await?;
                text = "".to_string();
            }
        }
        self.send_message(chat_id, &text).await
    }

    // Send list of venues available at given location
    pub async fn send_available_venues_at_location(&self, chat_id: i64, location_code: &str, venues: Vec<(String, String)>) -> Result<(), RequestError> {
        let mut text = format!("Available venues at {} are: \n\n",location_code);
        for (i, venue) in venues.iter().enumerate(){
            let venue_text = format!("{}. {} - <pre>{}</pre>\n\n",i+1, venue.0, venue.1);
            text.push_str(&venue_text.to_string());

            // If length of text exceed 3500 (limit is 4096), send it and start a new one
            if text.len() > 3500 {
                self.send_message(chat_id, &text).await?;
                text = "".to_string();
            }
        }
        self.send_message(chat_id, &text).await
    }

    // Send waiting list of a given user
    pub async fn send_waiting_list_for_user(&self, chat_id: i64, waiting_list: Vec<(String, String, String)>) -> Result<(), RequestError> {
        let mut text = "Your waiting list is: \n\n".to_string();
        for (i, wl) in waiting_list.iter().enumerate(){
            let wl_text = format!("{}. {} at {} on {}\n\n",i+1, wl.0, wl.1, wl.2);
            text.push_str(&wl_text.to_string());

            // If length of text exceed 3500 (limit is 4096), send it and start a new one
            if text.len() > 3500 {
                self.send_message(chat_id, &text).await?;
                text = "".to_string();
            }
        }
        self.send_message(chat_id, &text).await
    }

    // Send help message to user mentioning commands and their usage
    pub async fn send_help_message(&self, chat_id: i64) -> Result<(), RequestError> {
        let mut text = "Available commands are:\n\n \
                /wl - Get your waiting list\n\n \
                /list_locations - List all available locations\n\n \
                /list_venues <location_code> - List all available venues at given location\n\n \
                /enroll <movie_code> <venue_code> <date_string> - Enroll for notification for given movie at given venue on given date\n\n \
                - Example Usage: /enroll ET00310790 PVKC 22-04-2021\n\n \
                - Venue Code can be obtained using above list_venues command\n\n \
                - Date string should be in DD-MM-YYYY format\n\n \
                - Movie Code is present in the URL of the movie's page on in.bookmyshow.com. \n\n \
                - Sample URL with movie code at end: https://in.bookmyshow.com/kochi/movies/spider-man-no-way-home/ET00310790\n\n \
                NOTE : Each user can have a maximum of 2 enrollments"
            .to_string();
        text = self.escape_out_special_characters(&text);
        self.send_message(chat_id, &text).await
    }

    pub async fn send_limited_enrollments_message(&self, chat_id: i64) -> Result<(), RequestError> {
        let mut text = "Sorry! It seems you are already waiting for maximum allowed shows. \n\n\
                        Due to large number of users we can't let you enroll for more. \
                        You can enroll for more, once these shows start booking or the date is past. \n\n\
                        Rest assured, you will be notified as soon as booking opens for the shows you enrolled for. So technically you are \
                        already guaranteed to get your favourite seats at your favourite venue 🙃."
                    .to_string();
        text = self.escape_out_special_characters(&text);
        self.send_message(chat_id, &text).await
    }

    // Incorrect Request
    pub async fn _incorrect_request(
        &self,
        chat_id: i64,
    ) -> Result<(), RequestError> {
        self.send_message(chat_id, "Incorrect Request").await
    }

    fn escape_out_special_characters(&self, text: &str) -> String {
        // Telegram API currently supports only the following named HTML entities:
        // &lt;, &gt;, &amp; and &quot;.
        // These have to be escaped out before sending to prevent error
        text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    }
}