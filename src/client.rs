use crate::error::{Error, Result};
use scraper::{Html, Selector};
use std::collections::BTreeSet;

fn fetch_session() -> Result<(String, String)> {
    let url = "http://autorace.jp/netstadium/SearchRace";
    let mut response = reqwest::get(url)?;
    let session_id = response
        .cookies()
        .find(|cookie| cookie.name() == "PHPSESSID")
        .ok_or_else(|| Error::CookieError)?
        .value()
        .to_string();
    let html = response.text()?;
    let token = Html::parse_document(&html)
        .select(&Selector::parse("input#search_race__token").unwrap())
        .next()
        .ok_or_else(|| Error::HtmlParseError)?
        .value()
        .attr("value")
        .ok_or_else(|| Error::HtmlParseError)?
        .to_string();
    Ok((session_id, token))
}

fn fetch_result_of_month(
    year: usize,
    month: usize,
    session_id: &str,
    token: &str,
) -> Result<Vec<String>> {
    let year_from = year;
    let year_to = if month == 12 { year + 1 } else { year };
    let month_from = month;
    let month_to = if month == 12 { 1 } else { month + 1 };

    let url = "http://autorace.jp/netstadium/SearchRace/Result";
    let params = [
        ("search_race[lg]", "0".to_string()),
        ("search_race[date_from][year]", year_from.to_string()),
        ("search_race[date_from][month]", month_from.to_string()),
        ("search_race[date_from][day]", 1.to_string()),
        ("search_race[date_to][year]", year_to.to_string()),
        ("search_race[date_to][month]", month_to.to_string()),
        ("search_race[date_to][day]", 1.to_string()),
        ("search_race[paragraph]", "".to_string()),
        ("search_race[_token]", token.to_string()),
    ];

    let tr_selector = Selector::parse("tr").unwrap();
    let table_selector = Selector::parse("table#tblRace").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let client = reqwest::Client::new();
    let html = client
        .post(url)
        .form(&params)
        .header("Cookie", &format!("PHPSESSID={}", session_id))
        .send()?
        .text()?;
    Html::parse_document(&html)
        .select(&table_selector)
        .next()
        .ok_or_else(|| Error::HtmlParseError)?
        .select(&tr_selector)
        .skip(1)
        .map(|tr| {
            Ok(tr
                .select(&td_selector)
                .next()
                .ok_or_else(|| Error::HtmlParseError)?
                .select(&a_selector)
                .next()
                .ok_or_else(|| Error::HtmlParseError)?
                .value()
                .attr("href")
                .ok_or_else(|| Error::HtmlParseError)?
                .to_string())
        })
        .collect::<Result<Vec<_>>>()
}

pub fn fetch_result_urls_of_year(year: usize) -> Result<BTreeSet<String>> {
    let (session_id, token) = fetch_session()?;
    Ok((1..13)
        .map(|month| fetch_result_of_month(year, month, &session_id, &token))
        .collect::<Result<Vec<Vec<String>>>>()?
        .into_iter()
        .flat_map(|x| x)
        .collect::<BTreeSet<String>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fetch_result_urls_of_year() {
        let url_set = fetch_result_urls_of_year(2018);
        assert_eq!(url_set.unwrap().len(), 463);
    }
}
