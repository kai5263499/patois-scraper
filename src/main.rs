use reqwest;
use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};
use serde_json::{json, to_string_pretty};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use tokio; 

#[derive(Serialize, Deserialize, Debug)]
struct WordEntry {
    word: String,
    definition: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LostWordEntry {
    word: String,
    part_of_speech: String,
    years: String,
    definition: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut all_words = Vec::new();
    let td_selector = Selector::parse("td").unwrap();  // Create the Selector outside the loop

    let args: Vec<String> = std::env::args().collect();
    let scrape_all_words = args.contains(&"--scrape-all-words".to_string());
    let scrape_lost_words = args.contains(&"--scrape-lost-words".to_string());

    if scrape_all_words {
        
        for letter in 'a'..='z' {
            let url = format!("https://phrontistery.info/{}.html", letter);
            println!("Scraping {}", url);

            let resp = reqwest::get(&url).await?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await?;
                let (text, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
                let document = Html::parse_document(&text);
                let table_selector = Selector::parse("table.words tbody").unwrap();
                let row_selector = Selector::parse("tr").unwrap();

                for tbody in document.select(&table_selector) {
                    for row in tbody.select(&row_selector) {
                        let mut tds = row.select(&td_selector);  // Use the existing Selector
                        if let (Some(word), Some(definition)) = (tds.next(), tds.next()) {
                            let word = word.text().collect::<Vec<_>>().join("").trim().replace("\n", "").to_string();
                            let definition = definition.text().collect::<Vec<_>>().join("").trim().replace("\n", "").to_string();
                            
                            if word == "Word" {
                                continue;
                            }

                            all_words.push(WordEntry {
                                word: word,
                                definition: definition,
                            });
                        }
                    }
                }
            } else {
                eprintln!("Failed to retrieve {}", url);
            }
        }
    }

    let mut all_lost_words = Vec::new();

    let table_selector = Selector::parse("table.list tbody").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("th, td").unwrap();

    if scrape_lost_words {
        for i in 1..=4 {
            let url = format!("https://phrontistery.info/clw{}.html", i);
            println!("Scraping {}", url);

            let resp = reqwest::get(&url).await?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await?;
                let (text, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
                
                let document = Html::parse_document(&text);

                for tbody in document.select(&table_selector) {
                    let mut rows = tbody.select(&row_selector);
                    while let Some(row) = rows.next() {
                        let mut tds = row.select(&td_selector);
                        
                        if let (Some(word), Some(part_of_speech), Some(years)) = (tds.next(), tds.next(), tds.next()) {
                            if let (Some(definition_row), Some(description_row)) = (rows.next(), rows.next()) {
                                let definition = definition_row.text().collect::<Vec<_>>().join("");
                                let description = description_row.text().collect::<Vec<_>>().join("");
                                all_lost_words.push(LostWordEntry {
                                    word: word.text().collect::<Vec<_>>().join("").trim().to_string(),
                                    part_of_speech: part_of_speech.text().collect::<Vec<_>>().join("").trim().to_string(),
                                    years: years.text().collect::<Vec<_>>().join("").trim().replace("\n", "").to_string(),
                                    definition: definition.trim().to_string(),
                                    description: description.trim().to_string(),
                                });
                            }
                        }
                    }
                }
                
            } else {
                println!("Failed to retrieve {}", url);  // Debug print
            }
        }
    }

    let json_data = json!({ "allWords": all_words, "lostWords": all_lost_words });

    let file_name = "pronthist.json";
    let mut file = File::create(file_name)?;
    writeln!(file, "{}", to_string_pretty(&json_data)?)?;
    println!("Data saved to {}", file_name);

    Ok(())
}
