//! Interfaces related to search feature only. For more information, see [`SearchClient`].

pub(crate) mod macros;
mod query;
mod selectors;

use scraper::{Html, Selector};
use serde::Deserialize;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::{
    error::Result,
    interface::product::{AgeCategory, WorkType},
    utils::ToParseError,
    DlsiteClient,
    cache::GenericCache,
};

pub use self::query::SearchProductQuery;

/// Client to search products on DLsite.
pub struct SearchClient<'a> {
    pub(crate) c: &'a DlsiteClient,
    /// Cache for search results to avoid re-parsing the same queries
    result_cache: Arc<Mutex<GenericCache<Vec<SearchProductItem>>>>,
}

#[derive(Deserialize)]
struct SearchPageInfo {
    count: i32,
}

#[derive(Deserialize)]
struct SearchAjaxResult {
    search_result: String,
    page_info: SearchPageInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SearchProductItem {
    pub id: String,
    pub title: String,
    pub creator: Option<String>,
    pub creator_omitted: Option<bool>,
    pub circle_name: String,
    pub circle_id: String,
    pub dl_count: Option<i32>,
    pub rate_count: Option<i32>,
    pub review_count: Option<i32>,
    pub price_original: i32,
    pub price_sale: Option<i32>,
    pub age_category: AgeCategory,
    pub work_type: WorkType,
    pub thumbnail_url: String,
    pub rating: Option<f32>, // pub image_url: Option<String>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub products: Vec<SearchProductItem>,
    pub count: i32,
    pub query_path: String,
}
fn parse_count_str(str: &str) -> Result<i32> {
    str.replace(['(', ')', ','], "")
        .parse()
        .to_parse_error("Failed to parse string to count")
}

fn parse_num_str(str: &str) -> Result<i32> {
    str.replace(',', "")
        .parse()
        .to_parse_error("Failed to parse string to number")
}

impl<'a> SearchClient<'a> {
    /// Create a new search client
    pub(crate) fn new(c: &'a DlsiteClient) -> Self {
        Self {
            c,
            result_cache: Arc::new(Mutex::new(GenericCache::new(100, Duration::from_secs(3600)))),
        }
    }

    /// Search products on DLsite.
    ///
    /// # Arguments
    /// * `options` - Struct of search options.
    ///
    /// # Example
    /// ```
    /// use dlsite::{DlsiteClient, client::search::SearchProductQuery, interface::query::*};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let product = client
    ///         .search()
    ///         .search_product(&SearchProductQuery {
    ///             sex_category: Some(vec![SexCategory::Male]),
    ///             keyword: Some("ASMR".to_string()),
    ///             ..Default::default()
    ///         })
    ///         .await
    ///         .expect("Failed to search");
    ///     dbg!(&product);
    /// }
    /// ```
    pub async fn search_product(&self, options: &SearchProductQuery) -> Result<SearchResult> {
        let query_path = options.to_path();

        // Check if results are cached
        {
            let cache = self.result_cache.lock().unwrap();
            if let Some(cached_products) = cache.get(&query_path) {
                // Get count from API (it's small and fast)
                let json = self.c.get(&query_path).await?;
                let json = serde_json::from_str::<SearchAjaxResult>(&json)?;
                let count = json.page_info.count;

                return Ok(SearchResult {
                    products: cached_products,
                    count,
                    query_path,
                });
            }
        }

        // Cache miss - fetch and parse
        let json = self.c.get(&query_path).await?;
        let json = serde_json::from_str::<SearchAjaxResult>(&json)?;
        let html = json.search_result;
        let count = json.page_info.count;

        // Use parallel parsing for better performance
        let products = parse_search_html_parallel(&html)?;

        // Cache the results
        {
            let cache = self.result_cache.lock().unwrap();
            cache.insert(query_path.clone(), products.clone());
        }

        Ok(SearchResult {
            products,
            count,
            query_path,
        })
    }

    /// Search multiple queries concurrently for better performance
    /// This method uses tokio::join_all to fetch multiple pages in parallel
    ///
    /// # Arguments
    /// * `queries` - Vector of search queries to execute
    ///
    /// # Returns
    /// * `Vec<SearchResult>` - Results for each query in the same order
    pub async fn search_products_batch(&self, queries: &[SearchProductQuery]) -> Result<Vec<SearchResult>> {
        let futures: Vec<_> = queries
            .iter()
            .map(|q| self.search_product(q))
            .collect();

        futures::future::try_join_all(futures).await
    }

    /// Stream search results for a query, parsing items as they are fetched
    /// This method is optimized for memory efficiency and responsiveness
    ///
    /// # Arguments
    /// * `options` - Search query options
    /// * `callback` - Function to call for each parsed item
    ///
    /// # Returns
    /// * `Result<i32>` - Total count of items
    pub async fn search_product_stream<F>(&self, options: &SearchProductQuery, mut callback: F) -> Result<i32>
    where
        F: FnMut(SearchProductItem),
    {
        let query_path = options.to_path();
        let json = self.c.get(&query_path).await?;
        let json = serde_json::from_str::<SearchAjaxResult>(&json)?;
        let html = json.search_result;
        let count = json.page_info.count;

        // Parse and stream items
        let html = Html::parse_fragment(&html);
        for item_element in html.select(&Selector::parse("#search_result_img_box > li").unwrap()) {
            let item_html = item_element.html();
            match parse_search_item_html(&item_html) {
                Ok(item) => callback(item),
                Err(e) => eprintln!("Warning: Failed to parse item: {:?}", e),
            }
        }

        Ok(count)
    }
}

/// Parse a single search result item from HTML element
/// This function is designed to be used in parallel processing
fn parse_search_item_html(item_html: &str) -> Result<SearchProductItem> {
    let item_element = Html::parse_fragment(item_html);
    let item_element = item_element
        .root_element();

    let product_id_e = item_element
        .select(selectors::product_id_element())
        .next()
        .to_parse_error("Failed to find data element")?
        .value();
    let maker_e = item_element
        .select(selectors::maker_name())
        .next()
        .to_parse_error("Failed to find maker element")?;
    let author_e = item_element
        .select(selectors::author())
        .next();

    let price_e = item_element
        .select(selectors::work_price())
        .next()
        .to_parse_error("Failed to find price element")?;
    let original_price_e = item_element
        .select(selectors::original_price())
        .next();
    let (sale_price_e, original_price_e) = if let Some(e) = original_price_e {
        (Some(price_e), e)
    } else {
        (None, price_e)
    };
    let id = product_id_e
        .attr("data-product_id")
        .to_parse_error("Failed to get product id")?
        .to_string();

    Ok(SearchProductItem {
        id: id.clone(),
        title: item_element
            .select(selectors::work_title())
            .next()
            .to_parse_error("Failed to get title")?
            .value()
            .attr("title")
            .unwrap()
            .to_string(),
        age_category: {
            if let Some(e) = item_element
                .select(selectors::age_category())
                .next()
            {
                let title = e.value().attr("title");
                if let Some(title) = title {
                    match title {
                        "全年齢" => AgeCategory::General,
                        "R-15" => AgeCategory::R15,
                        _ => {
                            return Err(crate::DlsiteError::Parse(
                                "Age category parse error: invalid title".to_string(),
                            ))
                        }
                    }
                } else {
                    return Err(crate::DlsiteError::Parse(
                        "Age category parse error".to_string(),
                    ));
                }
            } else {
                AgeCategory::Adult
            }
        },
        circle_name: maker_e.text().next().unwrap_or("").to_string(),
        circle_id: maker_e
            .value()
            .attr("href")
            .to_parse_error("Failed to get maker link")?
            .split('/')
            .next_back()
            .to_parse_error("Invalid url")?
            .split('.')
            .next()
            .to_parse_error("Failed to find maker id")?
            .to_string(),
        creator: {
            if let Some(creator_e) = author_e {
                let name = creator_e
                    .select(selectors::creator_link())
                    .next()
                    .to_parse_error("Failed to find creator")?
                    .text()
                    .next()
                    .to_parse_error("Failed to find creator")?
                    .to_string();
                Some(name)
            } else {
                None
            }
        },
        creator_omitted: {
            if let Some(creator_e) = author_e {
                let omitted = creator_e
                    .value()
                    .attr("class")
                    .to_parse_error("Failed to find creator")?
                    .split(" ")
                    .any(|x| x == "omit");
                Some(omitted)
            } else {
                None
            }
        },
        dl_count: {
            if let Some(e) = item_element
                .select(selectors::dl_count())
                .next()
            {
                Some(
                    e.text()
                        .next()
                        .to_parse_error("Failed to get dl count")?
                        .replace(',', "")
                        .parse()
                        .to_parse_error("Invalid dl count")?,
                )
            } else {
                None
            }
        },
        rate_count: {
            if let Some(e) = item_element
                .select(selectors::dl_count())
                .next()
            {
                Some(parse_count_str(
                    e.text().next().to_parse_error("Failed to get rate count")?,
                )?)
            } else {
                None
            }
        },
        review_count: {
            if let Some(e) = item_element
                .select(selectors::review_count())
                .next()
            {
                Some(parse_count_str(
                    e.text()
                        .next()
                        .to_parse_error("Failed to get review count")?,
                )?)
            } else {
                None
            }
        },
        price_original: parse_num_str(
            original_price_e
                .text()
                .next()
                .to_parse_error("Failed to find price")?,
        )?,
        price_sale: {
            match sale_price_e {
                Some(e) => Some(parse_num_str(
                    e.text().next().to_parse_error("Failed to find price")?,
                )?),
                None => None,
            }
        },
        work_type: item_element
            .select(selectors::work_category())
            .next()
            .to_parse_error("Failed to find work category")?
            .value()
            .attr("class")
            .to_parse_error("Failed to find worktype")?
            .split(' ')
            .find_map(|c| {
                if let Some(c) = c.strip_prefix("type_") {
                    if let Ok(wt) = c.parse::<WorkType>() {
                        if let WorkType::Unknown(_) = wt {
                            return None;
                        } else {
                            return Some(wt);
                        }
                    }
                }
                None
            })
            .unwrap_or(WorkType::Unknown("".to_string())),
        thumbnail_url: {
            let img_e = item_element
                .select(selectors::thumbnail_image())
                .next()
                .to_parse_error("Failed to find thumbnail")?;

            let src = img_e.value().attr("src");
            let data_src = img_e.value().attr("data-src");
            match (src, data_src) {
                (Some(src), _) => format!("https:{}", src),
                (_, Some(data_src)) => format!("https:{}", data_src),
                (_, _) => {
                    return Err(crate::DlsiteError::Parse(
                        "Failed to find thumbnail".to_string(),
                    ))
                }
            }
        },
        rating: {
            if let Some(e) = item_element
                .select(selectors::rating())
                .next()
            {
                e.value()
                    .attr("class")
                    .expect("Failed to get rating")
                    .split(' ')
                    .find_map(|c| {
                        if let Some(c) = c.strip_prefix("star_") {
                            if let Ok(r) = c.parse::<f32>() {
                                return Some(r / 10.0);
                            }
                        }
                        None
                    })
            } else {
                None
            }
        },
    })
}

pub(crate) fn parse_search_html(html: &str) -> Result<Vec<SearchProductItem>> {
    let html = Html::parse_fragment(html);
    let mut result: Vec<SearchProductItem> = vec![];

    for item_element in html.select(&Selector::parse("#search_result_img_box > li").unwrap()) {
        let product_id_e = item_element
            .select(&Selector::parse("div[data-product_id]").unwrap())
            .next()
            .to_parse_error("Failed to find data element")?
            .value();
        let maker_e = item_element
            .select(&Selector::parse(".maker_name a").unwrap())
            .next()
            .to_parse_error("Failed to find maker element")?;
        let author_e = item_element
            .select(&Selector::parse(".author").unwrap())
            .next();

        let price_e = item_element
            .select(&Selector::parse(".work_price .work_price_base").unwrap())
            .next()
            .to_parse_error("Failed to find price element")?;
        let original_price_e = item_element
            .select(&Selector::parse(".work_price_wrap .strike .work_price_base").unwrap())
            .next();
        let (sale_price_e, original_price_e) = if let Some(e) = original_price_e {
            (Some(price_e), e)
        } else {
            (None, price_e)
        };
        let id = product_id_e
            .attr("data-product_id")
            .to_parse_error("Failed to get product id")?
            .to_string();

        result.push(SearchProductItem {
            id: id.clone(),
            title: item_element
                .select(&Selector::parse(".work_name a[title]").unwrap())
                .next()
                .to_parse_error("Failed to get title")?
                .value()
                .attr("title")
                .unwrap()
                .to_string(),
            age_category: {
                if let Some(e) = item_element
                    .select(&Selector::parse(".work_genre span").unwrap())
                    .next()
                {
                    let title = e.value().attr("title");
                    if let Some(title) = title {
                        match title {
                            "全年齢" => AgeCategory::General,
                            "R-15" => AgeCategory::R15,
                            _ => {
                                return Err(crate::DlsiteError::Parse(
                                    "Age category parse error: invalid title".to_string(),
                                ))
                            }
                        }
                    } else {
                        return Err(crate::DlsiteError::Parse(
                            "Age category parse error".to_string(),
                        ));
                    }
                } else {
                    AgeCategory::Adult
                }
            },
            circle_name: maker_e.text().next().unwrap_or("").to_string(),
            circle_id: maker_e
                .value()
                .attr("href")
                .to_parse_error("Failed to get maker link")?
                .split('/')
                .next_back()
                .to_parse_error("Invalid url")?
                .split('.')
                .next()
                .to_parse_error("Failed to find maker id")?
                .to_string(),
            creator: {
                if let Some(creator_e) = author_e {
                    let name = creator_e
                        .select(&Selector::parse("a").unwrap())
                        .next()
                        .to_parse_error("Failed to find creator")?
                        .text()
                        .next()
                        .to_parse_error("Failed to find creator")?
                        .to_string();
                    Some(name)
                } else {
                    None
                }
            },
            creator_omitted: {
                if let Some(creator_e) = author_e {
                    let omitted = creator_e
                        .value()
                        .attr("class")
                        .to_parse_error("Failed to find creator")?
                        .split(" ")
                        .any(|x| x == "omit");
                    Some(omitted)
                } else {
                    None
                }
            },
            dl_count: {
                if let Some(e) = item_element
                    .select(&Selector::parse(".work_dl span[class*=\"dl_count\"]").unwrap())
                    .next()
                {
                    Some(
                        e.text()
                            .next()
                            .to_parse_error("Failed to get dl count")?
                            .replace(',', "")
                            .parse()
                            .to_parse_error("Invalid dl count")?,
                    )
                } else {
                    None
                }
            },
            rate_count: {
                if let Some(e) = item_element
                    .select(&Selector::parse(".work_dl span[class*=\"dl_count\"]").unwrap())
                    .next()
                {
                    Some(parse_count_str(
                        e.text().next().to_parse_error("Failed to get rate count")?,
                    )?)
                } else {
                    None
                }
            },
            review_count: {
                if let Some(e) = item_element
                    .select(&Selector::parse(".work_review div a").unwrap())
                    .next()
                {
                    Some(parse_count_str(
                        e.text()
                            .next()
                            .to_parse_error("Failed to get review count")?,
                    )?)
                } else {
                    None
                }
            },
            price_original: parse_num_str(
                original_price_e
                    .text()
                    .next()
                    .to_parse_error("Failed to find price")?,
            )?,
            price_sale: {
                match sale_price_e {
                    Some(e) => Some(parse_num_str(
                        e.text().next().to_parse_error("Failed to find price")?,
                    )?),
                    None => None,
                }
            },
            work_type: item_element
                .select(&Selector::parse(".work_category").unwrap())
                .next()
                .to_parse_error("Failed to find work category")?
                .value()
                .attr("class")
                .to_parse_error("Failed to find worktype")?
                .split(' ')
                .find_map(|c| {
                    if let Some(c) = c.strip_prefix("type_") {
                        if let Ok(wt) = c.parse::<WorkType>() {
                            if let WorkType::Unknown(_) = wt {
                                return None;
                            } else {
                                return Some(wt);
                            }
                        }
                    }
                    None
                })
                .unwrap_or(WorkType::Unknown("".to_string())),
            thumbnail_url: {
                let img_e = item_element
                    .select(&Selector::parse(".work_thumb_inner > img").unwrap())
                    .next()
                    .to_parse_error("Failed to find thumbnail")?;

                let src = img_e.value().attr("src");
                let data_src = img_e.value().attr("data-src");
                match (src, data_src) {
                    (Some(src), _) => format!("https:{}", src),
                    (_, Some(data_src)) => format!("https:{}", data_src),
                    (_, _) => {
                        return Err(crate::DlsiteError::Parse(
                            "Failed to find thumbnail".to_string(),
                        ))
                    }
                }
            },
            rating: {
                if let Some(e) = item_element
                    .select(&Selector::parse(".work_rating .star_rating").unwrap())
                    .next()
                {
                    e.value()
                        .attr("class")
                        .expect("Failed to get rating")
                        .split(' ')
                        .find_map(|c| {
                            if let Some(c) = c.strip_prefix("star_") {
                                if let Ok(r) = c.parse::<f32>() {
                                    return Some(r / 10.0);
                                }
                            }
                            None
                        })
                } else {
                    None
                }
            }, // image_url: {
               //     if let Some(e) = item_element
               //         .select(&Selector::parse(".work_img_popover img").unwrap())
               //         .next()
               //     {
               //         Some(
               //             e.value()
               //                 .attr("src")
               //                 .to_parse_error("Failed to get image url")?
               //                 .to_string(),
               //         )
               //     } else {
               //         None
               //     }
               // },
        })
    }

    Ok(result)
}

/// Parse search HTML using parallel processing for better performance
/// This function is optimized for large result sets (50+ items)
pub(crate) fn parse_search_html_parallel(html: &str) -> Result<Vec<SearchProductItem>> {
    let html = Html::parse_fragment(html);

    // Collect all item elements as HTML strings
    let items: Vec<String> = html
        .select(&Selector::parse("#search_result_img_box > li").unwrap())
        .map(|elem| elem.html())
        .collect();

    // Process items in parallel
    items
        .par_iter()
        .map(|item_html| parse_search_item_html(item_html))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        client::DlsiteClient,
        interface::{
            product::WorkType,
            query::{Order, SexCategory},
        },
    };

    #[tokio::test]
    async fn search_product_1() {
        let client = DlsiteClient::default();
        let res = client
            .search()
            .search_product(&super::SearchProductQuery {
                sex_category: Some(vec![SexCategory::Male]),
                keyword: Some("ねこぐらし".to_string()),
                order: Some(Order::Release),
                ..Default::default()
            })
            .await
            .expect("Failed to search");

        assert!(res.products.len() >= 10);
        assert!(res.count >= 45);

        res.products
            .iter()
            .find(|r| r.id == "RJ291224")
            .expect("Expected to find RJ291224");

        res.products.iter().for_each(|r| {
            if r.id == "RJ291224" {
                assert_eq!(1980, r.price_original);
                assert!(r.dl_count.unwrap() > 9000);
                assert!(r.rate_count.is_some());
                assert!(r.review_count.is_some());
                assert!(r.rating.is_some());
                assert_eq!("RG51654", r.circle_id);
                assert_eq!("CANDY VOICE", r.circle_name);
                assert_eq!(WorkType::SOU, r.work_type);
                assert_eq!("竹達彩奈", r.creator.as_ref().unwrap());
                assert!(!r.creator_omitted.unwrap());
            }
        });
    }

    #[tokio::test]
    async fn search_product_2() {
        let client = DlsiteClient::default();
        let mut opts = super::SearchProductQuery {
            sex_category: Some(vec![SexCategory::Male]),
            order: Some(Order::Trend),
            per_page: Some(50),
            ..Default::default()
        };

        let res = client
            .search()
            .search_product(&opts)
            .await
            .expect("Failed to search page 1");
        res.products.iter().for_each(|i| {
            url::Url::parse(&i.thumbnail_url).expect("Failed to parse url");
            dbg!(&i);
        });
        assert_eq!(50, res.products.len());

        opts.page = Some(2);
        let res = client
            .search()
            .search_product(&opts)
            .await
            .expect("Failed to search page 2");
        res.products.iter().for_each(|i| {
            url::Url::parse(&i.thumbnail_url).expect("Failed to parse url");
        });
        assert_eq!(50, res.products.len());
    }
}
