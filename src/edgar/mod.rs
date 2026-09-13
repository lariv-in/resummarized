//! SEC EDGAR ingest plugin — Atom feeds from sec.gov.

pub mod apps;
pub mod atom;
pub mod config;
pub mod entities;
pub mod feeds;
pub mod fetch;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod query;
pub mod routes;
pub mod rune_env;
pub mod state;
pub mod stock_market;
pub mod templates;
pub mod workers;

use frunk::{HCons, HNil, hlist::HList};
use lariv_rs::{
    app::App,
    capability::CapStore,
    config::{ConfigCap, ConfigTag},
    db::{DbCap, DbTag},
    define_passthrough_cap, define_plugin_install,
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

use config::{EdgarConfig, EdgarConfigTag};
use state::EdgarState;

/// Plugin identity tag.
pub struct EdgarTag;

define_passthrough_cap!(EdgarStateCap, EdgarTag, EdgarState);

define_plugin_install! {
    plugin: EdgarTag;
    /// Register SEC EDGAR migrations, routes, templates, dashboard tile, and poller.
    steps: [
        cap_hook(crate::stock_markets::StockMarketTag, crate::stock_markets::StockMarketsCap, stock_market::Hook),
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(EdgarConfigTag, EdgarConfig),
        http(routes::Hook),
        state(StateHook),
        serve_startup(workers::ServeStartupHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, EdgarCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, EdgarCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<EdgarConfigTag, EdgarCfgIdx, Value = EdgarConfig>,
    L: HList + CapTagAbsent<EdgarTag, TagProof>,
{
    type Output = HCons<EdgarStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<EdgarConfigTag, EdgarCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let client = fetch::build_client().expect("EDGAR HTTP client");
        app.add_capability(CapStore::with_items(EdgarState::new(conn, config, client)))
    }
}
