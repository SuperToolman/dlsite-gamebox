/// Cached CSS selectors for search result parsing
/// This module provides pre-compiled selectors to avoid recompiling them on every parse

use scraper::Selector;
use std::sync::OnceLock;

/// Get the selector for search result items
pub fn search_result_items() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse("#search_result_img_box > li").expect("Failed to parse selector")
    })
}

/// Get the selector for product ID element
pub fn product_id_element() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse("div[data-product_id]").expect("Failed to parse selector")
    })
}

/// Get the selector for maker name
pub fn maker_name() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".maker_name a").expect("Failed to parse selector")
    })
}

/// Get the selector for author
pub fn author() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".author").expect("Failed to parse selector")
    })
}

/// Get the selector for work price
pub fn work_price() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_price .work_price_base").expect("Failed to parse selector")
    })
}

/// Get the selector for original price
pub fn original_price() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_price_wrap .strike .work_price_base").expect("Failed to parse selector")
    })
}

/// Get the selector for work title
pub fn work_title() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_name a[title]").expect("Failed to parse selector")
    })
}

/// Get the selector for age category
pub fn age_category() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_genre span").expect("Failed to parse selector")
    })
}

/// Get the selector for download count
pub fn dl_count() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_dl span[class*=\"dl_count\"]").expect("Failed to parse selector")
    })
}

/// Get the selector for review count
pub fn review_count() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_review div a").expect("Failed to parse selector")
    })
}

/// Get the selector for work category
pub fn work_category() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_category").expect("Failed to parse selector")
    })
}

/// Get the selector for thumbnail image
pub fn thumbnail_image() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_thumb_inner > img").expect("Failed to parse selector")
    })
}

/// Get the selector for rating
pub fn rating() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse(".work_rating .star_rating").expect("Failed to parse selector")
    })
}

/// Get the selector for creator link
pub fn creator_link() -> &'static Selector {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        Selector::parse("a").expect("Failed to parse selector")
    })
}

