//! Rune sandbox bindings for NSE RSS trigram search.

use std::sync::Arc;

use lariv_rs::rune_env::{
    NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar, block_on_async, json_to_rune,
};

use crate::nse::entities::item::{self, Entity as ItemEntity};
use crate::nse::entities::status::{self, Entity as StatusEntity};
use crate::search::{parse_search_args, results_json, search_models};

const ITEM_TEXT_COLUMNS: &[item::Column] = &[
    item::Column::FeedKind,
    item::Column::Title,
    item::Column::Description,
    item::Column::Subject,
    item::Column::Series,
    item::Column::Purpose,
    item::Column::FaceValue,
    item::Column::RelatingTo,
    item::Column::AuditedUnaudited,
    item::Column::Cumulative,
    item::Column::Consolidated,
    item::Column::IndAs,
    item::Column::Period,
    item::Column::EncumberedPromoterNames,
    item::Column::AcquirerNames,
    item::Column::PromoterNames,
    item::Column::FinancialYear,
    item::Column::SubmissionType,
    item::Column::Remarks,
    item::Column::BrsrNseSymbol,
    item::Column::BrsrScripCode,
    item::Column::BrsrMseiSymbol,
    item::Column::BrsrIsin,
    item::Column::BrsrCin,
    item::Column::BrsrCompanyName,
    item::Column::BrsrRegisteredOffice,
    item::Column::BrsrCorporateOffice,
    item::Column::BrsrEmail,
    item::Column::BrsrTelephone,
    item::Column::BrsrWebsite,
    item::Column::BrsrPaidUpCapital,
    item::Column::BrsrContactPerson,
    item::Column::BrsrContactPhone,
    item::Column::BrsrContactEmail,
    item::Column::BrsrReportingBoundary,
    item::Column::BrsrCoreAssurance,
    item::Column::BrsrTurnover,
    item::Column::BrsrNetWorth,
    item::Column::BrsrStatesServed,
    item::Column::BrsrCountriesServed,
    item::Column::BrsrBoardSize,
    item::Column::BrsrFemaleDirectors,
    item::Column::BrsrKmp,
    item::Column::BrsrFemaleKmp,
    item::Column::BrsrCsrApplicable,
    item::Column::UhpScripCode,
    item::Column::UhpNseSymbol,
    item::Column::UhpMseiSymbol,
    item::Column::UhpSebiRegistration,
    item::Column::UhpCompanyName,
    item::Column::UhpTypeOfReport,
    item::Column::UhpNumberOfSecurities,
    item::Column::VotingScripCode,
    item::Column::VotingSymbol,
    item::Column::VotingMseiSymbol,
    item::Column::VotingIsin,
    item::Column::VotingCompanyName,
    item::Column::VotingTypeOfMeeting,
    item::Column::VotingStartTime,
    item::Column::VotingEndTime,
    item::Column::VotingScrutinizer,
    item::Column::VotingScrutinizerFirm,
    item::Column::VotingScrutinizerQualification,
    item::Column::VotingScrutinizerMembership,
    item::Column::VotingShareholdersOnRecord,
    item::Column::VotingPromotersInPerson,
    item::Column::VotingPublicInPerson,
    item::Column::VotingPromotersVc,
    item::Column::VotingPublicVc,
    item::Column::VotingResolutionsPassed,
    item::Column::SodNseSymbol,
    item::Column::SodScripCode,
    item::Column::SodMseiSymbol,
    item::Column::SodIsin,
    item::Column::SodCompanyName,
    item::Column::SodStatementCount,
    item::Column::SodModeOfFundRaising,
    item::Column::SodAmountRaised,
    item::Column::SodMonitoringAgency,
    item::Column::SodMonitoringAgencyName,
    item::Column::SodHasDeviation,
    item::Column::SodDeviationExplanation,
    item::Column::SodShareholderApproved,
    item::Column::SodAuditCommitteeComments,
    item::Column::SodAuditorComments,
    item::Column::SodSignatory,
    item::Column::SodDesignation,
    item::Column::SodPlace,
    item::Column::ShpNseSymbol,
    item::Column::ShpScripCode,
    item::Column::ShpMseiSymbol,
    item::Column::ShpIsin,
    item::Column::ShpCompanyName,
    item::Column::ShpClassOfSecurity,
    item::Column::ShpTypeOfReport,
    item::Column::ShpFiledUnder,
    item::Column::ShpPromoterPct,
    item::Column::ShpPublicPct,
    item::Column::ShpPromoterShares,
    item::Column::ShpPublicShares,
    item::Column::ScrNseSymbol,
    item::Column::ScrScripCode,
    item::Column::ScrMseiSymbol,
    item::Column::ScrIsin,
    item::Column::ScrCompanyName,
    item::Column::ScrObservationsReported,
    item::Column::ScrPreviousObservations,
    item::Column::ScrActionsTaken,
    item::Column::ScrCertifyingFirm,
    item::Column::ScrPcsName,
    item::Column::ScrMembershipType,
    item::Column::ScrMembershipNumber,
    item::Column::ScrUdin,
    item::Column::ScrCpNumber,
    item::Column::ScrPlace,
    item::Column::RptNseSymbol,
    item::Column::RptScripCode,
    item::Column::RptMseiSymbol,
    item::Column::RptCompanyName,
    item::Column::RptReportingPeriod,
    item::Column::RptHasRelatedParty,
    item::Column::RptEnteredTransactions,
    item::Column::RptTransactionCount,
    item::Column::RptCounterparty,
    item::Column::RptTransactionType,
    item::Column::RptAmount,
    item::Column::IcNseSymbol,
    item::Column::IcScripCode,
    item::Column::IcMseiSymbol,
    item::Column::IcIsin,
    item::Column::IcCompanyName,
    item::Column::IcClass,
    item::Column::IcSubmissionType,
    item::Column::IcPendingStart,
    item::Column::IcReceived,
    item::Column::IcDisposed,
    item::Column::IcPendingEnd,
    item::Column::IcScoresId,
    item::Column::ItNseSymbol,
    item::Column::ItScripCode,
    item::Column::ItMseiSymbol,
    item::Column::ItIsin,
    item::Column::ItCompanyName,
    item::Column::ItRegulation,
    item::Column::ItInstrument,
    item::Column::ItPerson,
    item::Column::ItCategory,
    item::Column::ItTxnType,
    item::Column::ItQty,
    item::Column::ItValue,
    item::Column::ItMode,
    item::Column::ItPriorQty,
    item::Column::ItPriorPct,
    item::Column::ItPostQty,
    item::Column::ItPostPct,
    item::Column::ItSignatory,
    item::Column::ItDesignation,
    item::Column::ItExchange,
    item::Column::IffNseSymbol,
    item::Column::IffScripCode,
    item::Column::IffMseiSymbol,
    item::Column::IffIsin,
    item::Column::IffCompanyName,
    item::Column::IffTypeOfCompany,
    item::Column::IffClassOfSecurity,
    item::Column::IffReportingPeriod,
    item::Column::IffReportingQuarter,
    item::Column::IffAudited,
    item::Column::IffNature,
    item::Column::IffRevenue,
    item::Column::IffProfit,
    item::Column::FrNseSymbol,
    item::Column::FrScripCode,
    item::Column::FrMseiSymbol,
    item::Column::FrCompanyName,
    item::Column::FrClassOfSecurity,
    item::Column::FrReportingQuarter,
    item::Column::FrAudited,
    item::Column::FrNature,
    item::Column::FrRevenue,
    item::Column::FrProfit,
];

const STATUS_TEXT_COLUMNS: &[status::Column] =
    &[status::Column::FeedKind, status::Column::LastError];

/// Registers NSE table search helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        rune_env.register_contextual(
            "search_nse_rss_items",
            "search_nse_rss_items(#{ query?: string, from?: string, to?: string, limit?: int }) -> #{ results: [NseRssItem] }  // trigram search across text columns; optional inclusive pub_date range (IST or RFC 3339); at least one of query/from/to",
            |_ctx| NativeBinding::Function(Arc::new(search_nse_rss_items)),
        );
        rune_env.register_contextual(
            "search_nse_feed_status",
            "search_nse_feed_status(#{ query?: string, from?: string, to?: string, limit?: int }) -> #{ results: [NseFeedStatus] }  // trigram search on feed_kind/last_error; optional inclusive last_fetched_at range (IST or RFC 3339); at least one of query/from/to",
            |_ctx| NativeBinding::Function(Arc::new(search_nse_feed_status)),
        );
    }
}

fn search_nse_rss_items(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed = parse_search_args("search_nse_rss_items", args)?;
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        search_models::<ItemEntity, _, _>(&db, ITEM_TEXT_COLUMNS, item::Column::PubDate, &parsed)
            .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(results_json(&rows))
}

fn search_nse_feed_status(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed = parse_search_args("search_nse_feed_status", args)?;
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        search_models::<StatusEntity, _, _>(
            &db,
            STATUS_TEXT_COLUMNS,
            status::Column::LastFetchedAt,
            &parsed,
        )
        .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(results_json(&rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use lariv_rs::plugins::filesystem::storage::{DynFilestore, UnimplementedFilestore};
    use sea_orm::DatabaseConnection;
    use serde_json::json;

    fn test_env_ctx<'a>(
        db: &'a DatabaseConnection,
        store: &'a Arc<DynFilestore>,
    ) -> RuneEnvCtx<'a> {
        RuneEnvCtx {
            db,
            store: Arc::clone(store),
            session_id: None,
        }
    }

    fn registered_env() -> RuneEnvCapability {
        let mut cap = RuneEnvCapability::new();
        Hook.register_rune_env(&mut cap);
        cap
    }

    fn call_err(name: &str, args: serde_json::Value) -> String {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
            .unwrap_or_else(|| panic!("{name}"));
        let arg = json_to_rune(args).expect("args");
        f(&env_ctx, &[arg]).expect_err("expected error")
    }

    #[test]
    fn registers_nse_search_bindings() {
        let names = registered_env().all_names();
        for expected in ["search_nse_rss_items", "search_nse_feed_status"] {
            assert!(
                names.iter().any(|name| name == expected),
                "expected {expected} in {names:?}"
            );
        }
    }

    #[test]
    fn search_nse_rss_items_rejects_empty_args() {
        let err = call_err("search_nse_rss_items", json!({}));
        assert!(err.contains("query"), "{err}");
    }

    #[test]
    fn search_nse_feed_status_rejects_invalid_to() {
        let err = call_err("search_nse_feed_status", json!({ "to": "not-a-date" }));
        assert!(err.contains("to"), "{err}");
    }
}
