use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    VoteSymbol,
    VoteScripCode,
    VoteMseiSymbol,
    VoteIsin,
    VoteCompanyName,
    VoteTypeOfMeeting,
    VoteDateOfMeeting,
    VoteStartTime,
    VoteEndTime,
    VoteDateOfRecord,
    VoteShareholdersOnRecord,
    VoteResolutionsPassed,
    VotePromotersInPerson,
    VotePublicInPerson,
    VotePromotersVc,
    VotePublicVc,
    VoteScrutinizerName,
    VoteScrutinizerFirm,
    VoteScrutinizerQualification,
    VoteScrutinizerMembership,
    VoteScrutinizerAppointed,
    VoteReportDate,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let text = [
            NseRssItems::VoteSymbol,
            NseRssItems::VoteScripCode,
            NseRssItems::VoteMseiSymbol,
            NseRssItems::VoteIsin,
            NseRssItems::VoteCompanyName,
            NseRssItems::VoteTypeOfMeeting,
            NseRssItems::VoteStartTime,
            NseRssItems::VoteEndTime,
            NseRssItems::VoteShareholdersOnRecord,
            NseRssItems::VoteResolutionsPassed,
            NseRssItems::VotePromotersInPerson,
            NseRssItems::VotePublicInPerson,
            NseRssItems::VotePromotersVc,
            NseRssItems::VotePublicVc,
            NseRssItems::VoteScrutinizerName,
            NseRssItems::VoteScrutinizerFirm,
            NseRssItems::VoteScrutinizerQualification,
            NseRssItems::VoteScrutinizerMembership,
        ];
        for col in text {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .add_column(ColumnDef::new(col).text())
                        .to_owned(),
                )
                .await?;
        }
        for col in [
            NseRssItems::VoteDateOfMeeting,
            NseRssItems::VoteDateOfRecord,
            NseRssItems::VoteScrutinizerAppointed,
            NseRssItems::VoteReportDate,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .add_column(ColumnDef::new(col).date())
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::VoteReportDate,
            NseRssItems::VoteScrutinizerAppointed,
            NseRssItems::VoteScrutinizerMembership,
            NseRssItems::VoteScrutinizerQualification,
            NseRssItems::VoteScrutinizerFirm,
            NseRssItems::VoteScrutinizerName,
            NseRssItems::VotePublicVc,
            NseRssItems::VotePromotersVc,
            NseRssItems::VotePublicInPerson,
            NseRssItems::VotePromotersInPerson,
            NseRssItems::VoteResolutionsPassed,
            NseRssItems::VoteShareholdersOnRecord,
            NseRssItems::VoteDateOfRecord,
            NseRssItems::VoteEndTime,
            NseRssItems::VoteStartTime,
            NseRssItems::VoteDateOfMeeting,
            NseRssItems::VoteTypeOfMeeting,
            NseRssItems::VoteCompanyName,
            NseRssItems::VoteIsin,
            NseRssItems::VoteMseiSymbol,
            NseRssItems::VoteScripCode,
            NseRssItems::VoteSymbol,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
