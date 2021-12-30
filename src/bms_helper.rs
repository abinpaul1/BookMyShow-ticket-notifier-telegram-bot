use crate::utils::{parse_date_string,get_date_code, get_timestamp_as_millisecond};
use std::env;
use reqwest::Error;

// Handles API calls to BMS private API and returns formatted response
pub struct BmsHelper{
    app_version: String,
    app_version_code: String,
    bms_id_prefix: String,
    token: String,
}

use log::{info,warn};

impl BmsHelper{
    pub fn new()->BmsHelper{
        let app_version = env::var("BMS_APP_VERSION").expect("BMS_APP_VERSION not set");
        let mut app_version_code = app_version.replace(".", "");
        app_version_code.push_str("0");
        BmsHelper{
            // TODO : Update app version, code - play store version
            app_version : app_version,
            app_version_code : app_version_code,
            bms_id_prefix : "1.58091598.".to_string(),
            token : "67x1xa33b4x422b361ba".to_string()
        }
    }

    // Functions that interact with API

    // Returns a vector of movie names
    pub async fn api_get_movies_at_venue(&self,venue_code: &str, date_str: &str) -> Result<Vec<String>, Error> {
        info!("BmsHelper::api_get_movies_at_venue: venue_code: {}, date_str: {}", venue_code, date_str);
        let date_code = get_date_code(&parse_date_string(date_str));

        // Create BMS ID
        let timestamp = get_timestamp_as_millisecond();
        let bms_id = format!("{}.{}", self.bms_id_prefix, timestamp);

        let url = format!(
            "https://in.bookmyshow.com/api/v2/mobile/showtimes/byvenue?appCode=MOBAND2&appVersion={}&venueCode={}&bmsId={}&token={}&dateCode={}",
            self.app_version_code,
            venue_code,
            bms_id,
            self.token,
            date_code
        );
        // Make request with user agent  headers
        let client = reqwest::Client::new();
        let headers = self.get_custom_headers(&bms_id, None);

        let resp = client
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let mut movies: Vec<String> = Vec::new();
        // Parse response and return vector of movie names
        if !resp["ShowDetails"].as_array().unwrap().is_empty(){
            for event in resp["ShowDetails"][0]["Event"].as_array().unwrap() {
                movies.push(event["EventTitle"].as_str().unwrap().to_string());
            }
        }
        else{
            warn!("ShowDetails was empty for venue: {}", venue_code);
        }
        Ok(movies)
    }

    // Returns a vector of tuple of venue name and venue code
    pub async fn api_get_venues_for_region(&self, region_code: &str) -> Result<Vec<(String,String)>, Error> {
        info!("BmsHelper::api_get_venues_for_region: {}", region_code);

        let url = format!(
            "https://in.bookmyshow.com/pwa/api/de/venues?regionCode={}&eventType=MT",
            region_code,
        );
        // Make request with user agent  headers
        let client = reqwest::Client::new();

        let resp = client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Parse response and return a vector of tuple of venue name and venue code
        let mut venues: Vec<(String, String)> = Vec::new();
        for venue in resp["BookMyShow"]["arrVenue"].as_array().unwrap() {
            let venue_name = venue["VenueName"].as_str().unwrap().to_string();
            let venue_code = venue["VenueCode"].as_str().unwrap().to_string();
            venues.push((venue_name, venue_code));
        }

        Ok(venues)
    }

    // Returns movie name for a given movie code, or None if invalid
    pub async fn api_get_movie_name_from_code(&self, movie_code: &str) -> Result<Option<String>, Error>{        
        info!("BmsHelper::api_get_movie_name_from_code: movie_code: {}", movie_code);
        let region_code = "BANG".to_string(); // Hardcoding region code

        // Create BMS ID
        let timestamp = get_timestamp_as_millisecond();
        let bms_id = format!("{}.{}", self.bms_id_prefix, timestamp);

        let url = format!(
        "https://in.bookmyshow.com/api/movies/v1/synopsis/init?eventcode={}&channel=mobile",
            movie_code
        );

        let client = reqwest::Client::new();
        let headers = self.get_custom_headers(&bms_id, Some(&region_code));

        let resp = client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        // Check status code for success
        if resp.status().is_success() {
            let resp_json = resp.json::<serde_json::Value>().await?;
            let movie_name = resp_json["meta"]["event"]["eventName"].as_str().unwrap().to_string();
            return Ok(Some(movie_name));
        }

        warn!("Failed to get movie name from code: {}", movie_code);
        warn!("Response: {:#?}", resp);

        Ok(None)
    }

    // Returns venue name for a given venue code, or None if invalid
    pub async fn api_get_venue_name_from_code(&self, venue_code:&str) -> Result<Option<String>, Error>{
        // Create BMS ID
        let timestamp = get_timestamp_as_millisecond();
        let bms_id = format!("{}.{}", self.bms_id_prefix, timestamp);

        let url = format!(
        "https://in.bookmyshow.com/api/movies/v1/cinema/showcase?vc={}",
            venue_code
        );

        let client = reqwest::Client::new();
        let headers = self.get_custom_headers(&bms_id, None);

        let resp = client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        if resp.status().is_success() {
            let resp_json = resp.json::<serde_json::Value>().await?;
            let venue_name = resp_json["data"]["venueName"].as_str().unwrap().to_string();
            return Ok(Some(venue_name));
        }

        warn!("Failed to get venue name from code: {}", venue_code);
        warn!("Response: {:#?}", resp);

        Ok(None)
    }

        // Returns vector of tuple of location names and location code
    pub async fn api_get_all_locations(&self) -> Result<Vec<(String, String)>, Error>{
        // Currently hardcoding top locations
        // All locations > 1.5k
        let mut locations : Vec<(String, String)> = Vec::new();
        
        locations.extend(vec![
            ("Mumbai".to_string(), "MUMBAI".to_string()),
            ("National Capital Region (NCR)".to_string(), "NCR".to_string()),
            ("Bengaluru".to_string(), "BANG".to_string()),
            ("Hyderabad".to_string(), "HYD".to_string()),
            ("Ahmedabad".to_string(), "AHD".to_string()),
            ("Chandigarh".to_string(), "CHD".to_string()),
            ("Pune".to_string(), "PUNE".to_string()),
            ("Chennai".to_string(), "CHEN".to_string()),
            ("Kolkata".to_string(), "KOLK".to_string()),
            ("Kochi".to_string(), "KOCH".to_string())
        ]);
        Ok(locations)
    }


    // Utility functions

    // Returns a reqwest::header object with the custom headers
    fn get_custom_headers(&self, bms_id: &str, region_code: Option<&str>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        // Add multiple headers together
        headers.extend(
            vec![
                (reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("Dalvik/2.1.0 (Linux; U; Android 10; Google Pixel 3a Build/QQ1D.200105.002)")),
                (reqwest::header::HeaderName::from_static("x-bms-id"), reqwest::header::HeaderValue::from_str(bms_id).unwrap()),
                (reqwest::header::HeaderName::from_static("x-platform"), reqwest::header::HeaderValue::from_static("AND")),
                (reqwest::header::HeaderName::from_static("x-platform-code"), reqwest::header::HeaderValue::from_static("ANDROID")),
                (reqwest::header::HeaderName::from_static("x-app-code"), reqwest::header::HeaderValue::from_static("MOBAND2")),
                (reqwest::header::HeaderName::from_static("x-device-cake"), reqwest::header::HeaderValue::from_static("Android-Google Pixel 3a")),
                (reqwest::header::HeaderName::from_static("x-screen-height"), reqwest::header::HeaderValue::from_static("2094")),
                (reqwest::header::HeaderName::from_static("x-screen-width"), reqwest::header::HeaderValue::from_static("1080")),
                (reqwest::header::HeaderName::from_static("x-screen-density"), reqwest::header::HeaderValue::from_static("2.625")),
                (reqwest::header::HeaderName::from_static("x-app-version"), reqwest::header::HeaderValue::from_str(&self.app_version).unwrap()),
                (reqwest::header::HeaderName::from_static("x-app-version-code"), reqwest::header::HeaderValue::from_str(&self.app_version_code).unwrap()),
                (reqwest::header::HeaderName::from_static("x-network"), reqwest::header::HeaderValue::from_static("Android | WIFI")),
                (reqwest::header::HeaderName::from_static("x-latitude"), reqwest::header::HeaderValue::from_static("0.0")),
                (reqwest::header::HeaderName::from_static("x-longitude"), reqwest::header::HeaderValue::from_static("0.0")),
            ]
        );
        // Add region code if provided
        if let Some(region_code) = region_code{
            headers.insert(reqwest::header::HeaderName::from_static("x-region-code"), reqwest::header::HeaderValue::from_str(region_code).unwrap());
            headers.insert(reqwest::header::HeaderName::from_static("x-subregion-code"), reqwest::header::HeaderValue::from_str(region_code).unwrap());
        }
        headers
    }

}