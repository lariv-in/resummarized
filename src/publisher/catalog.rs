//! Subscriber filter catalogs. Exchange choices come from the stock-market capability.

use serde::Serialize;

use crate::bse::feeds::BseFeedKind;
use crate::edgar::feeds::EdgarFeedKind;
use crate::euronext::feeds::EuronextFeedKind;
use crate::jpx::feeds::JpxFeedKind;
use crate::nasdaq::feeds::NasdaqFeedKind;
use crate::nse::feeds::NseFeedKind;
use crate::stock_markets::{self, StockMarketRegistry};

use super::entities::subscriber::{NewsletterInterval, json_to_string_list};

#[derive(Clone, Copy, Debug)]
pub struct EventTypeChoice {
    pub exchange_key: &'static str,
    pub exchange_label: &'static str,
    pub slug: &'static str,
    pub label: &'static str,
}

pub fn event_types() -> Vec<EventTypeChoice> {
    let mut out = Vec::new();
    for kind in NseFeedKind::ALL {
        out.push(EventTypeChoice {
            exchange_key: "nse",
            exchange_label: "NSE",
            slug: kind.slug(),
            label: kind.display_name(),
        });
    }
    for kind in BseFeedKind::ALL {
        out.push(EventTypeChoice {
            exchange_key: "bse",
            exchange_label: "BSE",
            slug: kind.slug(),
            label: kind.display_name(),
        });
    }
    for kind in NasdaqFeedKind::ALL {
        out.push(EventTypeChoice {
            exchange_key: "nasdaq",
            exchange_label: "Nasdaq",
            slug: kind.slug(),
            label: kind.display_name(),
        });
    }
    for kind in EdgarFeedKind::ALL {
        out.push(EventTypeChoice {
            exchange_key: "edgar",
            exchange_label: "SEC EDGAR",
            slug: kind.slug(),
            label: kind.display_name(),
        });
    }
    for kind in JpxFeedKind::ALL {
        out.push(EventTypeChoice {
            exchange_key: "jpx",
            exchange_label: "JPX",
            slug: kind.slug(),
            label: kind.display_name(),
        });
    }
    for kind in EuronextFeedKind::ALL {
        out.push(EventTypeChoice {
            exchange_key: "euronext",
            exchange_label: "Euronext",
            slug: kind.slug(),
            label: kind.display_name(),
        });
    }
    out
}

pub fn newsletter_interval_choices() -> Vec<(String, String)> {
    NewsletterInterval::ALL
        .iter()
        .map(|interval| (interval.as_str().to_string(), interval.label().to_string()))
        .collect()
}

/// `(key, label)` pairs for admin exchange inputs.
pub fn exchange_choices() -> Vec<(String, String)> {
    stock_markets::choice_pairs()
}

#[derive(Clone, Debug, Serialize)]
pub struct ExchangeOption {
    pub key: String,
    pub label: String,
}

/// JSON options for the public subscribe combobox.
pub fn exchange_options() -> Vec<ExchangeOption> {
    stock_markets::markets()
        .iter()
        .map(|market| ExchangeOption {
            key: market.key.to_string(),
            label: market.label.to_string(),
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EventTypeOption {
    pub key: String,
    pub label: String,
    pub exchange: String,
}

/// JSON options for the public subscribe combobox.
pub fn event_type_options() -> Vec<EventTypeOption> {
    event_types()
        .into_iter()
        .map(|item| EventTypeOption {
            key: item.slug.to_string(),
            label: format!("{} ({})", item.label, item.exchange_label),
            exchange: item.exchange_key.to_string(),
        })
        .collect()
}

/// Filter event types by the given stock exchange keys (e.g. `["nse", "bse"]`).
/// If `exchanges` is empty, all event type options are returned.
pub fn event_type_options_for_exchanges(exchanges: &[String]) -> Vec<EventTypeOption> {
    if exchanges.is_empty() {
        return event_type_options();
    }
    event_types()
        .into_iter()
        .filter(|item| {
            exchanges
                .iter()
                .any(|ex| ex.eq_ignore_ascii_case(item.exchange_key))
        })
        .map(|item| EventTypeOption {
            key: item.slug.to_string(),
            label: format!("{} ({})", item.label, item.exchange_label),
            exchange: item.exchange_key.to_string(),
        })
        .collect()
}

pub fn format_exchange_display(
    value: &Option<sea_orm::prelude::Json>,
    markets: &StockMarketRegistry,
) -> String {
    let items = json_to_string_list(value);
    if items.is_empty() {
        "All".into()
    } else {
        items
            .iter()
            .map(|key| markets.label_for(key).to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_options_contain_exchange() {
        let options = event_type_options();
        assert!(!options.is_empty());
        assert!(options.iter().any(|opt| opt.exchange == "nse"));
        assert!(options.iter().any(|opt| opt.exchange == "nasdaq"));
    }

    #[test]
    fn event_type_options_filter_by_exchange() {
        let nse_only = event_type_options_for_exchanges(&["nse".into()]);
        assert!(!nse_only.is_empty());
        assert!(nse_only.iter().all(|opt| opt.exchange == "nse"));

        let multi = event_type_options_for_exchanges(&["nse".into(), "nasdaq".into()]);
        assert!(!multi.is_empty());
        assert!(multi.iter().all(|opt| opt.exchange == "nse" || opt.exchange == "nasdaq"));

        let empty = event_type_options_for_exchanges(&[]);
        assert_eq!(empty.len(), event_type_options().len());
    }
}
