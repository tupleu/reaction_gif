use std::fs::OpenOptions;

use regex::Regex;
use reqwest::Client;
use serde_json::{from_str, Value};

#[tokio::main]
async fn main() {
    let token = get_token().await;

    let url = "https://api.twitter.com/2/tweets/search/recent?query=url%3A%22https%3A%2F%2Ftenor.com%2Fview%22+-is%3Aretweet+-is%3Areply+lang%3Aen&tweet.fields=entities".to_string();
    let mut next_token = None;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("reaction_gifs.csv")
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);

    loop {
        let json = get_twitter(&token, &url, next_token).await;
        // next_token = Some(json["meta"]["next_token"].as_str().unwrap().to_owned());
        match json["meta"]["next_token"].as_str() {
            Some(x) => next_token = Some(x.to_owned()),
            None => break
        }

        let tenor_id_re = Regex::new(r"tenor\.com/view/.*?(\d+)(\?.*|&.*|$)").unwrap();
        let mut i = 0;
        while !json["data"][i].is_null() {
            let unwound_url = json["data"][i]["entities"]["urls"][0]["unwound_url"].as_str();
            if unwound_url == None {
                println!("ERROR: No Unwound_url");
                i += 1;
                continue;
            }
            if tenor_id_re.captures(unwound_url.unwrap()).is_none() {
                println!("ERROR: {}\n", unwound_url.unwrap());
                i += 1;
                continue;
            }
            let tweet_id = json["data"][i]["id"].as_str().unwrap();
            let gif_id = &tenor_id_re.captures(unwound_url.unwrap()).unwrap()[1];

            // Removes urls from text
            let remove_url_re = Regex::new(r"https://t.co.*?(\s|$)").unwrap();
            let text = remove_url_re.replace_all(json["data"][i]["text"].as_str().unwrap(), "").replace("\n", " ");

            // Gets tags from tenor
            let tags = get_tenor(gif_id).await.unwrap_or(String::from(""));
            
            println!("{}\n{}\n{}\n{:?}\n{:?}\n", tweet_id, unwound_url.unwrap(), gif_id, text, tags);

            match wtr.write_record(&[tweet_id, gif_id, &text, &tags]) {
                Ok(()) => (),
                Err(_) => println!("Error")
            };
            match wtr.flush() {
                Ok(()) => (),
                Err(_) => println!("Error")
            };

            i += 1;
        }
    }
}

async fn get_token() -> String {
    let credentials = std::fs::read_to_string("./credentials.txt").expect("should read token file");
    let mut iter = credentials.split_whitespace();
    let username = iter.next().unwrap().to_owned();
    let password = iter.next().unwrap().to_owned();
    // println!("{}, {}", username, password);
    let url = "https://api.twitter.com/oauth2/token";
    let params = [("grant_type","client_credentials")];
    let response = Client::new()
        .post(url)
        .basic_auth(username.clone(), Some(password.clone()))
        .form(&params)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let json: Value = from_str(std::str::from_utf8(&response).unwrap()).unwrap();
    json["access_token"].as_str().unwrap().to_owned()
}

async fn get_twitter(token: &String, url: &String, next_token: Option<String>) -> Value {
    let next = match next_token {
        Some(x) => format!("&next_token={}",x),
        None => String::from("")
    };
    let response = Client::new()
        .get(format!("{}{}",url, next))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let json: Value = from_str(std::str::from_utf8(&response).unwrap()).unwrap();
    json
}

async fn get_tenor(gif_id: &str) -> Option<String> {
    let url = format!("https://tenor.googleapis.com/v2/posts?ids={}&key=AIzaSyCFrrRS7AVUbYhPa63uilW4iMhOSY2KdYI", gif_id);
    let response = Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let json: Value = from_str(std::str::from_utf8(&response).unwrap()).unwrap();

    let tags = json["results"][0]["tags"].as_array();
    if tags.is_none() {
        return None;
    }
    let tags: String = tags.unwrap().into_iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .reduce(|a , e| format!("{}, {}",a,e)).unwrap();
    return Some(tags);
}
