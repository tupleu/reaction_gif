use std::fs;
use std::{fs::OpenOptions, io::Error};
use std::collections::HashMap;

use regex::Regex;
use reqwest::{Client, header::{ACCEPT, HeaderMap, USER_AGENT}};
use serde_json::{from_str, Value};

const person_token: &str = "[PERSON]";

#[tokio::main]
async fn main() {
    let twitter_token = get_twitter_token().await;
    let tenor_token = get_tenor_token();
    // let giphy_token = get_giphy_token();

    
    // tenor(&twitter_token, &tenor_token).await;
    // giphy(&twitter_token).await;
    // filter("data/");
    count_tags("filter.csv");
    // let tags = Vec::from(["happy","sad"])
    // let tags = Vec::from(["dance","happy","friday","clapping","clap","reaction","love","cute","party","weather","dog","funny","truth","applause","cheers","dancing","applaud","cat","animals","drinking","slow clap","bye","celebrate","angry","yay","laughing","smile","excited","weekend","movie","celebration","christmas","fun","thanksgiving","reactions","saturday","girl","monday","coffee","sad","friends","laugh","sunday","tgif","flower","rain","movies","puppy","wednesday","sun","crying","hello","yes","heart","snow","thursday","nice","no","alien","morning","yolo","storm","meme","wave","wow","art","clown","tuesday","congrats","fail","summer","swimming","good","robot","blessed","hot","turkey","cartoon","cry","fall","bunny","kitten","mad","funny animals","night","tv","life","surprise","i love you","day","omg","adorable","dogs","slap","beach","food","election","joker","interesting","love you","fire","winter","cool","good day","win","america","water","great job","mood","excellent","hug","space","hi","sports","crazy","haha","tired","what","yawn"])
        // .iter().map(|&s|s.into()).collect();
    // filter_tags("filter.csv", tags);
    let pos = ["happy", "love", "funny", "cute", "laughing", "smile", "party", "excited", "laugh", "yay", "yes", "hello", "fun", "celebration", "heart", "hug", "good morning", "i love you", "good", "kiss", "clap", "greeting", "clapping", "love you", "haha", "cheers", "thank you", "hi", "yeah", "thanks", "good night", "kawaii", "applause", "goodnight", "nice", "hahaha", "happy dance", "win", "lmao"].iter().map(|&s|s.into()).collect();
    let neg = ["angry", "sad", "no", "crying", "cry", "disaster", "middle finger", "tired", "fight", "fuck you", "wtf", "annoyed", "punch", "stupid", "nope", "kick", "facepalm", "annoying", "upset", "disgusted", "shit", "fuck", "burn", "poop", "fu", "fuck off", "spit", "bullshit", "frustrated", "oh no", "liar", "rage", "tears", "ass", "sobbing", "creepy", "oops", "stop", "go fuck yourself", "awkward", "death", "sob", "gross", "anger", "dwight crying", "lying", "shame", "trash", "boo", "stressed", "vomit", "tantrum", "exhausted", "evil"].iter().map(|&s|s.into()).collect();
    sentiment_analysis("filter.csv", pos, neg);
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

    let tags: String = json["results"][0]["tags"].as_array()?.into_iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .reduce(|a , e| format!("{}, {}",a,e))?;
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

fn get_kv(tweet: csv::StringRecord) -> Option<(String, (String, String, String))> {
    let key = tweet.get(0)?.to_owned();
    let value = (tweet.get(1)?.to_owned(), tweet.get(2)?.to_owned(), tweet.get(3)?.to_owned());
    Some((key, value))
}

fn filter(folder: &str) -> Result<(),Error>{
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("filter.csv")
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    let paths = fs::read_dir(folder)?;

    let mut texts = HashMap::new();

    let replace_replies_re = Regex::new(r"@\S+").unwrap();
    let only_replies_re = Regex::new(r"^\s*(\[PERSON\]\s*)+$").unwrap();
    let beginning_replies_re = Regex::new(r"^\s*(\[PERSON\]\s*)+").unwrap();

    for p in paths {
        let path = p?.path();
        let mut tweets = HashMap::new();
        let mut rdr = csv::Reader::from_path(path)?;
        for result in rdr.records() {
            let tweet = result?;
            if let Some((key, value)) = get_kv(tweet) {
                tweets.insert(key, value);
            }
        }
        for tweet in tweets {
            let mut text = tweet.1.1;
            let tags = tweet.1.2;

            text = text.replace("New trending GIF tagged", "");
            text = text.replace("New GIF tagged", "");
            text = text.replace("via Giphy", "");
            text = text.replace("via @gifkeyboard", "");
            text = text.replace("vía @gifkeyboard", "");
            text = text.replace("using @gifkeyboard", "");
            text = text.replace("via @giphy", "");
            text = text.replace("vía @giphy", "");
            text = text.replace("via @GIPHY", "");
            text = text.replace("GIFs | Tenor", "");
            text = replace_replies_re.replace_all(&text, person_token).to_string();
            text = beginning_replies_re.replace_all(&text, "").to_string();
            text = text.trim().to_string();

            if only_replies_re.is_match(&text) {
                continue
            }
            if text.is_empty() {
                continue
            }
            if tags.is_empty() {
                continue
            }
            texts.insert(text,(tweet.0,tags));
            // for tag in tags.split(',') {
                // wtr.write_record(&[&tweet.0, &text, tag.trim()])?;
                // wtr.write_record(&[&text, tag.trim()])?;
                // wtr.flush()?;
            // }
        }
    }

    for (text,(id,tags)) in texts {
        for tag in tags.split(',') {
            wtr.write_record(&[&id, &text, tag.trim()])?;
            // wtr.write_record(&[&text, tag.trim()])?;
            wtr.flush()?;
        }
    }
    Ok(())
}

fn count_tags(path: &str) -> Result<(),Error> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut tags = HashMap::new();
    for result in rdr.records() {
        let tweet = result?;
        let tag = tweet.get(2).unwrap().to_owned().to_ascii_lowercase();
        if(tags.contains_key(&tag)) {
            let v = tags.get(&tag).unwrap();
            tags.insert(tag, v+1);
        }
        else {
            tags.insert(tag, 1);
        }
    }
    let mut tags = Vec::from_iter(tags.into_iter());
    tags.sort_by(|a, b| b.1.cmp(&a.1));
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("tags.csv")
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    for (tag, count) in tags {
        wtr.write_record(&[tag,count.to_string()])?;
        wtr.flush()?;
    }
    Ok(())
}

fn filter_tags(path: &str, tags: Vec<String>) -> Result<(),Error> {
    let mut rdr = csv::Reader::from_path(path)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("data.csv")
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    for result in rdr.records() {
        let tweet = result?;
        let text = tweet.get(1).unwrap().to_owned();
        let tag = tweet.get(2).unwrap().to_owned().to_ascii_lowercase();
        if !tags.contains(&tag) {
            continue
        }
        wtr.write_record(&[text, tags.iter().position(|r| r == &tag).unwrap().to_string()])?;
        wtr.flush()?;
    }
    Ok(())
}

fn sentiment_analysis(path: &str, pos: Vec<String>, neg: Vec<String>) -> Result<(),Error> {
    let mut rdr = csv::Reader::from_path(path)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("data.csv")
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    for result in rdr.records() {
        let tweet = result?;
        let text = tweet.get(1).unwrap().to_owned();
        let tag = tweet.get(2).unwrap().to_owned().to_ascii_lowercase();
        if pos.contains(&tag) {
            wtr.write_record(&[text, String::from("0")])?;
            wtr.flush()?;
        }
        else if neg.contains(&tag) {
            wtr.write_record(&[&text, &String::from("1")])?;
            // oversampling
            wtr.write_record(&[text, String::from("1")])?;
            wtr.flush()?;
        }
    }
    Ok(())
}