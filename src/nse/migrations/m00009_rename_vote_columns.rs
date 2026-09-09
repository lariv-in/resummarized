use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    VoteScripCode,
    VotingScripCode,
    VoteSymbol,
    VotingSymbol,
    VoteMseiSymbol,
    VotingMseiSymbol,
    VoteIsin,
    VotingIsin,
    VoteCompanyName,
    VotingCompanyName,
    VoteTypeOfMeeting,
    VotingTypeOfMeeting,
    VoteDateOfMeeting,
    VotingDateOfMeeting,
    VoteStartTime,
    VotingStartTime,
    VoteEndTime,
    VotingEndTime,
    VoteScrutinizerName,
    VotingScrutinizer,
    VoteScrutinizerFirm,
    VotingScrutinizerFirm,
    VoteScrutinizerQualification,
    VotingScrutinizerQualification,
    VoteScrutinizerMembership,
    VotingScrutinizerMembership,
    VoteScrutinizerAppointed,
    VotingBoardMeetingDate,
    VoteReportDate,
    VotingReportIssuanceDate,
    VoteDateOfRecord,
    VotingRecordDate,
    VoteShareholdersOnRecord,
    VotingShareholdersOnRecord,
    VotePromotersInPerson,
    VotingPromotersInPerson,
    VotePublicInPerson,
    VotingPublicInPerson,
    VotePromotersVc,
    VotingPromotersVc,
    VotePublicVc,
    VotingPublicVc,
    VoteResolutionsPassed,
    VotingResolutionsPassed,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (from, to) in [
            (NseRssItems::VoteScripCode, NseRssItems::VotingScripCode),
            (NseRssItems::VoteSymbol, NseRssItems::VotingSymbol),
            (NseRssItems::VoteMseiSymbol, NseRssItems::VotingMseiSymbol),
            (NseRssItems::VoteIsin, NseRssItems::VotingIsin),
            (NseRssItems::VoteCompanyName, NseRssItems::VotingCompanyName),
            (
                NseRssItems::VoteTypeOfMeeting,
                NseRssItems::VotingTypeOfMeeting,
            ),
            (
                NseRssItems::VoteDateOfMeeting,
                NseRssItems::VotingDateOfMeeting,
            ),
            (NseRssItems::VoteStartTime, NseRssItems::VotingStartTime),
            (NseRssItems::VoteEndTime, NseRssItems::VotingEndTime),
            (
                NseRssItems::VoteScrutinizerName,
                NseRssItems::VotingScrutinizer,
            ),
            (
                NseRssItems::VoteScrutinizerFirm,
                NseRssItems::VotingScrutinizerFirm,
            ),
            (
                NseRssItems::VoteScrutinizerQualification,
                NseRssItems::VotingScrutinizerQualification,
            ),
            (
                NseRssItems::VoteScrutinizerMembership,
                NseRssItems::VotingScrutinizerMembership,
            ),
            (
                NseRssItems::VoteScrutinizerAppointed,
                NseRssItems::VotingBoardMeetingDate,
            ),
            (
                NseRssItems::VoteReportDate,
                NseRssItems::VotingReportIssuanceDate,
            ),
            (NseRssItems::VoteDateOfRecord, NseRssItems::VotingRecordDate),
            (
                NseRssItems::VoteShareholdersOnRecord,
                NseRssItems::VotingShareholdersOnRecord,
            ),
            (
                NseRssItems::VotePromotersInPerson,
                NseRssItems::VotingPromotersInPerson,
            ),
            (
                NseRssItems::VotePublicInPerson,
                NseRssItems::VotingPublicInPerson,
            ),
            (NseRssItems::VotePromotersVc, NseRssItems::VotingPromotersVc),
            (NseRssItems::VotePublicVc, NseRssItems::VotingPublicVc),
            (
                NseRssItems::VoteResolutionsPassed,
                NseRssItems::VotingResolutionsPassed,
            ),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .rename_column(from, to)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (from, to) in [
            (NseRssItems::VotingScripCode, NseRssItems::VoteScripCode),
            (NseRssItems::VotingSymbol, NseRssItems::VoteSymbol),
            (NseRssItems::VotingMseiSymbol, NseRssItems::VoteMseiSymbol),
            (NseRssItems::VotingIsin, NseRssItems::VoteIsin),
            (NseRssItems::VotingCompanyName, NseRssItems::VoteCompanyName),
            (
                NseRssItems::VotingTypeOfMeeting,
                NseRssItems::VoteTypeOfMeeting,
            ),
            (
                NseRssItems::VotingDateOfMeeting,
                NseRssItems::VoteDateOfMeeting,
            ),
            (NseRssItems::VotingStartTime, NseRssItems::VoteStartTime),
            (NseRssItems::VotingEndTime, NseRssItems::VoteEndTime),
            (
                NseRssItems::VotingScrutinizer,
                NseRssItems::VoteScrutinizerName,
            ),
            (
                NseRssItems::VotingScrutinizerFirm,
                NseRssItems::VoteScrutinizerFirm,
            ),
            (
                NseRssItems::VotingScrutinizerQualification,
                NseRssItems::VoteScrutinizerQualification,
            ),
            (
                NseRssItems::VotingScrutinizerMembership,
                NseRssItems::VoteScrutinizerMembership,
            ),
            (
                NseRssItems::VotingBoardMeetingDate,
                NseRssItems::VoteScrutinizerAppointed,
            ),
            (
                NseRssItems::VotingReportIssuanceDate,
                NseRssItems::VoteReportDate,
            ),
            (NseRssItems::VotingRecordDate, NseRssItems::VoteDateOfRecord),
            (
                NseRssItems::VotingShareholdersOnRecord,
                NseRssItems::VoteShareholdersOnRecord,
            ),
            (
                NseRssItems::VotingPromotersInPerson,
                NseRssItems::VotePromotersInPerson,
            ),
            (
                NseRssItems::VotingPublicInPerson,
                NseRssItems::VotePublicInPerson,
            ),
            (NseRssItems::VotingPromotersVc, NseRssItems::VotePromotersVc),
            (NseRssItems::VotingPublicVc, NseRssItems::VotePublicVc),
            (
                NseRssItems::VotingResolutionsPassed,
                NseRssItems::VoteResolutionsPassed,
            ),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .rename_column(from, to)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
