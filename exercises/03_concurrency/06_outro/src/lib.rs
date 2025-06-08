use pyo3::{prelude::*, types::PySet};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
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
//
// ──────────────────────────────────────────────
//  Public PyO3 entry point (GIL layer)
// ──────────────────────────────────────────────
//
pub fn site_map(start_from: String, site_map: Bound<'_, PySet>) {
    todo!()
}

#[pymodule]
fn outro3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(site_map, m)?)?;
    Ok(())
}

//
// ──────────────────────────────────────────────
//  Core Crawler Logic
// ──────────────────────────────────────────────
//
pub trait Fetcher {
    fn fetch(&self, url: &str) -> Option<String>;
}

pub struct HttpFetcher;

impl Fetcher for HttpFetcher {
    fn fetch(&self, url: &str) -> Option<String> {
        ureq::get(url).call().ok()?.into_string().ok()
    }
}

pub fn crawl_site<F: Fetcher + Sync>(start: &Url, fetcher: &F) -> HashSet<String> {
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let (work_tx, work_rx) = crossbeam::channel::unbounded::<String>();
    let (result_tx, result_rx) = crossbeam::channel::unbounded::<HashSet<String>>();
    let start_url = normalize_url(start);

    {
        let mut visited_guard = visited.lock().unwrap();
        visited_guard.insert(start_url.clone());
    }

    work_tx.send(start_url).unwrap();
    let mut n_pending = 1;

    // Spawn workers
    crossbeam::scope(|scope| {
        for _ in 0..num_cpus::get() {
            let work_rx = work_rx.clone();
            let result_tx = result_tx.clone();
            let fetcher = fetcher;

            scope.spawn(move |_| {
                while let Ok(current_url) = work_rx.recv() {
                    let links = if let Some(html) = fetcher.fetch(&current_url) {
                        let current_url = Url::parse(&current_url).unwrap();
                        extract_links_from_html(&current_url, &html)
                    } else {
                        HashSet::new()
                    };

                    result_tx.send(links).unwrap();
                }
            });
        }

        drop(result_tx);

        // Main orchestrator loop
        while n_pending > 0 {
            if let Ok(links) = result_rx.recv() {
                n_pending -= 1;

                for link in links {
                    let is_new_link = {
                        let mut visited_guard = visited.lock().unwrap();
                        visited_guard.insert(link.clone())
                    };

                    if is_new_link {
                        work_tx.send(link).unwrap();
                        n_pending += 1;
                    }
                }
            }
        }
    })
    .unwrap();

    Arc::try_unwrap(visited).unwrap().into_inner().unwrap()
}

fn extract_links_from_html(base_url: &Url, html: &str) -> HashSet<String> {
    let mut result = HashSet::new();
    let document = Html::parse_document(html.trim());
    let selector = Selector::parse(r#"a[href]"#).unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(joined) = base_url.join(href) {
                if joined.domain() == base_url.domain() {
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

//
// ──────────────────────────────────────────────
//  Unit & Integration Tests
// ──────────────────────────────────────────────
//
#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

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

        let result = extract_links_from_html(&base_url, html);

        let expected: HashSet<String> =
            vec!["http://example.com/about", "http://example.com/contact"]
                .into_iter()
                .map(String::from)
                .collect();

        assert_eq!(result, expected);
    }

    struct MockFetcher {
        pages: HashMap<String, String>,
    }

    impl Fetcher for MockFetcher {
        fn fetch(&self, url: &str) -> Option<String> {
            self.pages.get(url).cloned()
        }
    }

    #[test]
    fn it_crawls_multiple_pages_recursively() {
        let mut mock_pages = HashMap::new();

        mock_pages.insert(
            normalize_url(&Url::parse("http://a.com").unwrap()),
            r#"<a href="/page1">Page 1</a>"#.to_owned(),
        );
        mock_pages.insert(
            normalize_url(&Url::parse("http://a.com/page1").unwrap()),
            r#"<a href="/page2">Page 2</a>"#.to_owned(),
        );
        mock_pages.insert(
            normalize_url(&Url::parse("http://a.com/page2").unwrap()),
            "".to_owned(),
        );

        let mock_fetcher = MockFetcher { pages: mock_pages };

        let start = Url::parse("http://a.com").unwrap();
        let result = crawl_site(&start, &mock_fetcher);

        let expected: HashSet<String> =
            vec!["http://a.com/", "http://a.com/page1", "http://a.com/page2"]
                .into_iter()
                .map(String::from)
                .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn it_fetches_real_page() {
        let start = Url::parse("https://example.com").unwrap();
        let fetcher = HttpFetcher;

        let result = crawl_site(&start, &fetcher);

        assert!(result.contains("https://example.com/"));
    }
}
