//! Stock-market registry capability — exchange apps register at install time.
//!
//! The hub attaches [`StockMarketsCap`]; NSE/BSE/Nasdaq/JPX/Euronext add entries with
//! `cap_hook(StockMarketTag, StockMarketsCap, Hook)`. The publisher plugin resolves the
//! mounted [`StockMarketRegistry`] into form choices. Handlers extract
//! [`lariv_rs::http::Cap`]`<`[`StockMarketRegistry`]`>`.

use std::marker::PhantomData;
use std::sync::OnceLock;

use frunk::{HCons, HNil, hlist::HList};
use lariv_rs::{
    app::App,
    capability::{CapHookExt, Capability, HasCapTag},
    plugin_install::define_plugin_install,
    tag::Tagged,
    traits::add::{AddCapability, CapTagAbsent},
};

static MARKETS: OnceLock<Vec<StockMarket>> = OnceLock::new();

/// Capability tag for the stock-market registry.
pub struct StockMarketTag;

/// Plugin identity for the hub that attaches the empty registry.
pub struct StockMarketsHubTag;

define_plugin_install! {
    plugin: StockMarketsHubTag;
    /// Attach the stock-market registry so exchange apps can register themselves.
    steps: [
        cap_attach(StockMarketTag, StockMarketsCap, StockMarketsCap::<frunk::HNil>::new()),
        cap_hook(StockMarketTag, StockMarketsCap, BaseHook),
    ]
}

/// One exchange that subscriber filters may select.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StockMarket {
    pub key: &'static str,
    pub label: &'static str,
    pub order: u16,
}

/// Folded list of registered stock markets (mounted capability value).
#[derive(Clone, Debug, Default)]
pub struct StockMarketRegistry {
    markets: Vec<StockMarket>,
}

impl StockMarketRegistry {
    pub fn new() -> Self {
        Self {
            markets: Vec::new(),
        }
    }

    /// Register a market; duplicate keys are ignored (install-order first wins).
    pub fn register(mut self, market: StockMarket) -> Self {
        if !self
            .markets
            .iter()
            .any(|existing| existing.key == market.key)
        {
            self.markets.push(market);
        }
        self
    }

    pub fn markets(&self) -> &[StockMarket] {
        &self.markets
    }

    pub fn contains(&self, key: &str) -> bool {
        let key = key.trim();
        self.markets.iter().any(|market| market.key == key)
    }

    pub fn label_for<'a>(&'a self, key: &'a str) -> &'a str {
        self.markets
            .iter()
            .find(|market| market.key == key)
            .map(|market| market.label)
            .unwrap_or(key)
    }

    /// Keep only registered keys, preserving first-seen order.
    pub fn retain_known(&self, keys: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
        let mut out = Vec::new();
        for key in keys {
            let key = key.as_ref().trim();
            if key.is_empty() || !self.contains(key) {
                continue;
            }
            if !out.iter().any(|existing: &String| existing == key) {
                out.push(key.to_string());
            }
        }
        out
    }

    pub fn choice_pairs(&self) -> Vec<(String, String)> {
        self.sorted_markets()
            .into_iter()
            .map(|market| (market.key.to_string(), market.label.to_string()))
            .collect()
    }

    fn sorted_markets(&self) -> Vec<StockMarket> {
        let mut markets = self.markets.clone();
        markets.sort_by_key(|market| (market.order, market.key));
        markets
    }
}

/// Plugin hook for registering stock markets at install time.
pub trait StockMarketRegistrar: Sized {
    fn register_stock_markets(self, registry: StockMarketRegistry) -> StockMarketRegistry;
}

/// Builder-phase stock-market capability.
#[derive(Clone, Default)]
pub struct StockMarketsCap<Hooks> {
    pub hooks: Hooks,
    pub items: StockMarketRegistry,
    _tag: PhantomData<fn() -> StockMarketTag>,
}

impl<Hooks> StockMarketsCap<Hooks> {
    pub fn new() -> Self
    where
        Hooks: Default,
    {
        Self {
            hooks: Hooks::default(),
            items: StockMarketRegistry::new(),
            _tag: PhantomData,
        }
    }

    pub fn add_hook<HTag, H>(self, hook: H) -> StockMarketsCap<HCons<Tagged<HTag, H>, Hooks>> {
        StockMarketsCap {
            hooks: HCons {
                head: Tagged::new(hook),
                tail: self.hooks,
            },
            items: self.items,
            _tag: PhantomData,
        }
    }

    /// Eagerly fold registrar hooks into items (testing / pre-mount inspection).
    pub fn resolve_hooks(self) -> StockMarketsCap<HNil>
    where
        Hooks: FoldStockMarketRegistrarHooks,
    {
        let items = self.hooks.fold(self.items);
        StockMarketsCap {
            hooks: HNil,
            items,
            _tag: PhantomData,
        }
    }
}

impl<Hooks> HasCapTag for StockMarketsCap<Hooks> {
    type Tag = StockMarketTag;
}

impl<Hooks, Plugin, Hook> CapHookExt<Plugin, Hook> for StockMarketsCap<Hooks> {
    type Hooked = StockMarketsCap<HCons<Tagged<Plugin, Hook>, Hooks>>;

    fn prepend_cap_hook(self, hook: Hook) -> Self::Hooked {
        self.add_hook::<Plugin, Hook>(hook)
    }
}

/// Fold registrar hooks over the registry (tail first = install order).
pub trait FoldStockMarketRegistrarHooks {
    fn fold(self, registry: StockMarketRegistry) -> StockMarketRegistry;
}

impl FoldStockMarketRegistrarHooks for HNil {
    fn fold(self, registry: StockMarketRegistry) -> StockMarketRegistry {
        registry
    }
}

impl<Plugin, H, Tail> FoldStockMarketRegistrarHooks for HCons<Tagged<Plugin, H>, Tail>
where
    Tail: FoldStockMarketRegistrarHooks,
    H: StockMarketRegistrar + Copy,
{
    fn fold(self, registry: StockMarketRegistry) -> StockMarketRegistry {
        let registry = self.tail.fold(registry);
        self.head.value.register_stock_markets(registry)
    }
}

impl<Hooks> Capability for StockMarketsCap<Hooks>
where
    Hooks: FoldStockMarketRegistrarHooks,
{
    type Value = StockMarketRegistry;
    type Output = Tagged<StockMarketTag, StockMarketRegistry>;
    type Hooks = Hooks;
    type Items = StockMarketRegistry;

    fn mount(self) -> Self::Output {
        let registry = self.hooks.fold(self.items);
        let sorted = registry.sorted_markets();
        if MARKETS.set(sorted).is_err() {
            tracing::error!("stock market MARKETS already initialized");
        }
        Tagged::new(registry)
    }
}

/// No-op base hook from the hub plugin.
#[derive(Clone, Copy, Default)]
pub struct BaseHook;

impl StockMarketRegistrar for BaseHook {
    fn register_stock_markets(self, registry: StockMarketRegistry) -> StockMarketRegistry {
        registry
    }
}

/// Attach an empty stock-market capability (prefer `cap_attach` in install steps).
pub fn with_stock_markets<L, Proof>(app: App<L>) -> App<HCons<StockMarketsCap<HNil>, L>>
where
    L: HList + CapTagAbsent<StockMarketTag, Proof>,
{
    app.add_capability(StockMarketsCap::<HNil>::new())
}

/// Sorted markets folded at mount. Empty before the app is mounted.
pub fn markets() -> &'static [StockMarket] {
    MARKETS
        .get()
        .map(|markets| markets.as_slice())
        .unwrap_or(&[])
}

/// Form choices `(key, label)` from the mounted registry.
pub fn choice_pairs() -> Vec<(String, String)> {
    markets()
        .iter()
        .map(|market| (market.key.to_string(), market.label.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NseTag;
    struct BseTag;

    #[derive(Copy, Clone)]
    struct NseHook;

    impl StockMarketRegistrar for NseHook {
        fn register_stock_markets(self, registry: StockMarketRegistry) -> StockMarketRegistry {
            registry.register(StockMarket {
                key: "nse",
                label: "NSE",
                order: 10,
            })
        }
    }

    #[derive(Copy, Clone)]
    struct BseHook;

    impl StockMarketRegistrar for BseHook {
        fn register_stock_markets(self, registry: StockMarketRegistry) -> StockMarketRegistry {
            registry.register(StockMarket {
                key: "bse",
                label: "BSE",
                order: 20,
            })
        }
    }

    #[test]
    fn resolve_hooks_folds_registered_markets() {
        let cap = StockMarketsCap::<HNil>::new()
            .add_hook::<NseTag, _>(NseHook)
            .add_hook::<BseTag, _>(BseHook)
            .resolve_hooks();

        let keys: Vec<_> = cap.items.markets().iter().map(|m| m.key).collect();
        assert_eq!(keys, ["nse", "bse"]);
        assert!(cap.items.contains("nse"));
        assert_eq!(cap.items.label_for("bse"), "BSE");
        assert_eq!(
            cap.items.retain_known(["bse", "unknown", "nse", "bse"]),
            ["bse", "nse"]
        );
    }
}
