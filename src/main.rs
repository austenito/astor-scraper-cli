extern crate reqwest;
extern crate select;

use scraper::{Html, Selector};
use std::collections::HashMap;
use url::{ParseError, Url};

const ASTOR_URL: &str = "https://www.astorwines.com/WineSearchResult.aspx?search=Advanced&searchtype=Contains&term=&cat=1&saleitemsonly=True&Country=France&Color=White";
// &country=France&saleitemsonly=True&color=White&Page=1";

fn main() {
    fetch_astor_links(ASTOR_URL);
}

fn fetch_astor_links(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // https://www.astorwines.com/WineSearchResult.aspx?p=5&search=Advanced&searchtype=Contains&term=&cat=1&country=France&saleitemsonly=True&color=White&Page=3

    let mut done = false;
    let mut current_page = 1;
    let last_page = get_last_page(url);

    while !done {
        let mut current_url = Url::parse(&url.to_owned()).unwrap();
        current_url
            .query_pairs_mut()
            .append_pair("Page", &current_page.to_string());
        println!("{}", current_url.as_str());
        let resp = reqwest::blocking::get(current_url)?;
        let body = resp.text().unwrap();
        let fragment = Html::parse_document(&body);

        let wines = Selector::parse(".item-teaser").unwrap();

        println!("Page: {}", current_page);
        for wine in fragment.select(&wines) {
            println!("{}", get_text(&wine, ".header .item-name a"));
            let item_name = wine
                .select(&Selector::parse(".header .item-name a").unwrap())
                .next()
                .unwrap();
            let wine_link = item_name.value().attr("href").unwrap();
            println!("{}/{}", "https://www.astorwines.com", wine_link);

            println!(
                "Original: {}",
                get_text(&wine, ".price-value.price-old.price-bottle").trim()
            );
            println!("Sale: {}", get_text(&wine, ".price-sale").trim());
            println!("{}", get_text(&wine, ".price-bottle-discount").trim());
            println!(
                "{}",
                get_text(&wine, ".item-meta.supporting-text span").trim()
            );
            println!();
        }
        println!();

        current_page += 1;
        if current_page > last_page {
            done = true;
        }
        done = true;
    }
    Ok(())
}

fn get_last_page(url: &str) -> i32 {
    let resp = reqwest::blocking::get(url).unwrap();
    let body = resp.text().unwrap();
    let fragment = Html::parse_document(&body);

    let pagination_elements = Selector::parse(".pagination a:last-of-type").unwrap();
    let mut pagination_iterator = fragment.select(&pagination_elements);
    pagination_iterator.next();
    let input = pagination_iterator.next().unwrap();
    let last_page_link = input.value().attr("href").unwrap();
    println!("{}", last_page_link);

    let parsed_url = Url::parse(&(url.to_owned() + last_page_link)).unwrap();
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
    return hash_query.get("Page").unwrap().parse().unwrap();
}

fn get_text<'a>(element: &'a scraper::ElementRef, selector: &str) -> &'a str {
    let child = element
        .select(&Selector::parse(selector).unwrap())
        .next()
        .unwrap();
    return child.text().collect::<Vec<_>>()[0];
}
