use pyo3::{prelude::*, types::PySet};
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

#[pyfunction]
/// Given a starting URL (`start_from`), discover all the URLs *on the same domain*
/// that can be reached by following links from the starting URL.
///
/// The discovered URLs should be inserted into the `site_map` set provided as an argument.
///
/// # Constraints
///
/// ## GIL
///
/// You should, as much as possible, avoid holding the GIL.
/// Try to scope the GIL to the smallest possible block of code—e.g. when touching `site_map`.
///
/// ## Threads
///
/// The program should use as many threads as there are available cores on the machine.
///
/// ## Invalid URLs
///
/// If a URL is invalid (e.g. it's malformed or it returns a 404 status code), ignore it.
///
/// ## External URLs
///
/// Do not follow links to external websites. Restrict your search to the domain of the
/// starting URL.
///
/// ## Anchors and Query Parameters
///
/// Ignore anchors and query parameters when comparing URLs.
/// E.g. `http://example.com` and `http://example.com#section` should be considered the same URL,
/// and normalizing them to `http://example.com` is the expected approach.
///
/// # Tooling
///
/// We recommend using the following crates to help you with this exercise:
///
/// - `ureq` for making HTTP requests (https://crates.io/crates/ureq)
/// - `scraper` for parsing HTML and extracting links (https://crates.io/crates/scraper)
/// - `url` for parsing URLs (https://crates.io/crates/url)
/// - `std`'s `sync` and `thread` modules for synchronization primitives.
///
/// Feel free to pull in any other crates you think might be useful.
/// If your approach is channel-based, you might want to use the `crossbeam` crate too.
pub fn site_map(start_from: String, site_map: Bound<'_, PySet>) {
    todo!()
}

fn craw_with_mock(url: &Url, html: &str) -> HashSet<String> {
    let mut result = HashSet::new();
    let document = Html::parse_document(html.trim());
    let selector = Selector::parse(r#"a[href]"#).unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(joined) = url.join(href) {
                if joined.domain() == url.domain() {
                    let clean = normalize_url(&joined);
                    result.insert(clean);
                }
            };
        }
    }
    result
}

fn normalize_url(url: &Url) -> String {
    let mut url = url.clone();
    url.set_fragment(None);
    url.set_query(None);
    url.to_string()
}

#[pymodule]
fn outro3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(site_map, m)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_crawls_links_on_same_page() {
        let html = r#"
            <html>
                <body>
                    <a href="/about">About</a>
                    <a href="http://example.com/contact">Contact</a>
                    <a href="http://example.com/contact#section">Section</a>
                    <a href="http://other.com">External</a>
                </body>
            </html>
        "#;

        let base_url = Url::parse("http://example.com").unwrap();

        let result = craw_with_mock(&base_url, html);

        let expected: HashSet<String> =
            vec!["http://example.com/about", "http://example.com/contact"]
                .into_iter()
                .map(String::from)
                .collect();

        assert_eq!(result, expected);
    }
}
