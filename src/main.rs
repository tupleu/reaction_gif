use std::fs::OpenOptions;

use regex::Regex;
use reqwest::{Client, header::{ACCEPT, HeaderMap, USER_AGENT}};
use serde_json::{from_str, Value};

#[tokio::main]
async fn main() {
    let twitter_token = get_twitter_token().await;
    let tenor_token = get_tenor_token();
    // let giphy_token = get_giphy_token();

    
    tenor(&twitter_token, &tenor_token).await;
    giphy(&twitter_token).await;
}

async fn tenor(twitter_token: &str, tenor_token: &str) {
    let query = "tenor.com";
    let url = format!("https://api.twitter.com/2/tweets/search/recent?query={}+-is%3Aretweet&tweet.fields=entities", query).to_string();
    let mut next_token = None;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(format!("{}.csv",query))
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);

    let mut success_count = 0;
    let mut result_count = 0;

    let remove_url_re = Regex::new(r"https://t\.co.*?(\s|$)").unwrap();
    let tenor_id_re = Regex::new(r"tenor\.com[^&?\.]*?-(\d+)(?:\?|&|$)").unwrap();
    let media_tenor_id_re = Regex::new(r"\.tenor\.com/([^./]*?)/([^./]*?).gif").unwrap();

    loop {
        let json = get_twitter(twitter_token, &url, next_token).await;
        // println!("{:?}",json);
        // next_token = Some(json["meta"]["next_token"].as_str().unwrap().to_owned());
        match json["meta"]["next_token"].as_str() {
            Some(x) => next_token = Some(x.to_owned()),
            None => break
        }


        let mut i = 0;
        while !json["data"][i].is_null() {
            let unwound_url = json["data"][i]["entities"]["urls"][0]["unwound_url"].as_str();
            if unwound_url == None {
                println!("ERROR: No Unwound_url");
                i += 1;
                continue;
            }
            
            let text = remove_url_re.replace_all(json["data"][i]["text"].as_str().unwrap(), "").replace("\n", " ");
            let tweet_id = json["data"][i]["id"].as_str().unwrap();

            if tenor_id_re.captures(unwound_url.unwrap()).is_none() {

                if media_tenor_id_re.captures(unwound_url.unwrap()).is_none() {
                    println!("ERROR: {}\n", unwound_url.unwrap());
                    i += 1;
                    continue;
                }
                println!("SUCCESS2: {}\n", unwound_url.unwrap());
                let gif_id = &media_tenor_id_re.captures(unwound_url.unwrap()).unwrap()[1];
                let tags = media_tenor_id_re.captures(unwound_url.unwrap()).unwrap()[2].replace("-", ", ");

                match wtr.write_record(&[tweet_id, gif_id, &text, &tags]) {
                    Ok(()) => (),
                    Err(_) => println!("Error")
                };
                match wtr.flush() {
                    Ok(()) => (),
                    Err(_) => println!("Error")
                };
    
                success_count += 1;
                i += 1;
                continue;
            }
            println!("SUCCESS: {}\n", unwound_url.unwrap());
            let gif_id = &tenor_id_re.captures(unwound_url.unwrap()).unwrap()[1];

            // Gets tags from tenor
            let tags = get_tenor(&tenor_token,gif_id).await.unwrap_or(String::from(""));
            
            // println!("{}\n{}\n{}\n{:?}\n{:?}\n", tweet_id, unwound_url.unwrap(), gif_id, text, tags);

            match wtr.write_record(&[tweet_id, gif_id, &text, &tags]) {
                Ok(()) => (),
                Err(_) => println!("Error")
            };
            match wtr.flush() {
                Ok(()) => (),
                Err(_) => println!("Error")
            };

            success_count += 1;
            i += 1;
        }
        result_count += json["meta"]["result_count"].as_i64().unwrap();
        println!("oldest_id: {}", json["meta"]["oldest_id"].as_str().unwrap());
        println!("count: {}/{}", success_count, result_count);
    }
}

async fn giphy(twitter_token: &str) {
    let query = "giphy.com";
    let url = format!("https://api.twitter.com/2/tweets/search/recent?query={}+-is%3Aretweet&tweet.fields=entities", query).to_string();
    let mut next_token = None;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(format!("{}.csv",query))
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);

    let mut success_count = 0;
    let mut result_count = 0;

    let remove_url_re = Regex::new(r"https://t\.co.*?(\s|$)").unwrap();
    let giphy_url_re = Regex::new(r"giphy\.com/gifs").unwrap();
    let media_giphy_url_re = Regex::new(r"media\d*\.giphy\.com").unwrap();

    loop {
        let json = get_twitter(&twitter_token, &url, next_token).await;
        // println!("{:?}",json);
        // next_token = Some(json["meta"]["next_token"].as_str().unwrap().to_owned());
        match json["meta"]["next_token"].as_str() {
            Some(x) => next_token = Some(x.to_owned()),
            None => break
        }


        let mut i = 0;
        while !json["data"][i].is_null() {
            let unwound_url = json["data"][i]["entities"]["urls"][0]["unwound_url"].as_str();
            if unwound_url == None {
                println!("ERROR: No Unwound_url");
                i += 1;
                continue;
            }
            let mut giphy_url = unwound_url.unwrap().to_owned();
            if !giphy_url_re.is_match(&giphy_url) {
                
                if !media_giphy_url_re.is_match(&giphy_url) {
                    println!("ERROR: {}\n", giphy_url);
                    i += 1;
                    continue; 
                }

                println!("UNWRAPPING URL: {}\n", giphy_url);
                giphy_url = match get_media_giphy_url(giphy_url).await {
                    Some(x) => x,
                    _ => {i+=1; continue;}
                };
                // println!("HEEEEEEEEEERE")
            }
            
            let text = remove_url_re.replace_all(json["data"][i]["text"].as_str().unwrap(), "").replace("\n", " ");
            let tweet_id = json["data"][i]["id"].as_str().unwrap();

            println!("SUCCESS: {}\n", giphy_url);

            // Gets tags from giphy
            let tags = get_giphy(&giphy_url).await.unwrap_or(String::from(""));
            
            match wtr.write_record(&[tweet_id, &giphy_url, &text, &tags]) {
                Ok(()) => (),
                Err(_) => println!("Error")
            };
            match wtr.flush() {
                Ok(()) => (),
                Err(_) => println!("Error")
            };

            success_count += 1;
            i += 1;
        }
        result_count += json["meta"]["result_count"].as_i64().unwrap();
        println!("oldest_id: {}", json["meta"]["oldest_id"].as_str().unwrap());
        println!("count: {}/{}", success_count, result_count);
    }
   
}

async fn get_twitter_token() -> String {
    let credentials = std::fs::read_to_string("./twitter_credentials.txt").expect("should read token file");
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

fn get_tenor_token() -> String {
    let credentials = std::fs::read_to_string("./tenor_credentials.txt").expect("should read token file");
    let mut iter = credentials.split_whitespace();
    let key = iter.next().unwrap().to_owned();
    key
}

// fn get_giphy_token() -> String {
//     let credentials = std::fs::read_to_string("./giphy_credentials.txt").expect("should read token file");
//     let mut iter = credentials.split_whitespace();
//     let key = iter.next().unwrap().to_owned();
//     key
// }

async fn get_twitter(token: &str, url: &String, next_token: Option<String>) -> Value {
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

async fn get_tenor(key: &str, gif_id: &str) -> Option<String> {
    let url = format!("https://tenor.googleapis.com/v2/posts?ids={}&key={}", gif_id, key);
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

async fn get_giphy(url: &str) -> Option<String> {
    let response = Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let html = match std::str::from_utf8(&response) {
        Ok(x) => x,
        _ => return None
    };
    let get_html_tags_re = Regex::new(r#""tags.*?\[(.*?)\]"#).unwrap();
    let html_tags = get_html_tags_re.captures(html)?[1].to_owned();

    let mut tags = String::from("");
    let get_tags_re = Regex::new(r#""([^"]+)""#).unwrap();
    for cap in get_tags_re.captures_iter(&html_tags) {
        let tag = cap[1].to_owned();
        match tags.is_empty() {
            true => tags = tag,
            false => tags = format!("{}, {}", tags, tag)
        };
    }
    println!("{}", tags);
    match tags.is_empty() {
        true => None,
        false => Some(tags)
    }
}

async fn get_media_giphy_url(url: String) -> Option<String> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "text/html".parse().unwrap());
    headers.insert(USER_AGENT, "insomnia/2022.6.0".parse().unwrap());
    let response = Client::new()
        .get(url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let html = match std::str::from_utf8(&response) {
        Ok(x) => x,
        _ => return None
    };
    // println!("{:?}", html);
    let get_url_re = Regex::new(r#""(https://giphy\.com/gifs/.*?)""#).unwrap();
    let giphy_url = get_url_re.captures(html)?[1].to_owned();
    Some(giphy_url)
}

