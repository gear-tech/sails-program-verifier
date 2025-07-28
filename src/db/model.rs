use super::schema::{self, code::dsl as code_dsl, verification::dsl as verif_dsl};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::PgValue,
    prelude::{Insertable, Queryable},
    serialize::{IsNull, ToSql},
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
};
use serde::Serialize;
use std::{io::Write, time::SystemTime};
use utoipa::ToSchema;

#[derive(Queryable, Selectable, Insertable, Serialize, ToSchema)]
#[diesel(table_name = schema::code)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Code {
    pub id: String,
    pub idl_hash: Option<String>,
    pub name: String,
    pub repo_link: String,
}

impl Code {
    pub fn new(
        conn: &mut PgConnection,
        id: String,
        repo_link: String,
        name: String,
        idl_hash: Option<String>,
    ) -> Result<Code, diesel::result::Error> {
        let code = Code {
            id,
            idl_hash,
            name,
            repo_link,
        };

        diesel::insert_into(schema::code::table)
            .values(&code)
            .returning(Code::as_returning())
            .get_result(conn)
    }

    pub fn get(conn: &mut PgConnection, id: &str) -> Option<Code> {
        code_dsl::code.find(id).first(conn).ok()
    }

    pub fn get_many(
        conn: &mut PgConnection,
        ids: &[String],
    ) -> Result<Vec<Code>, diesel::result::Error> {
        code_dsl::code
            .filter(code_dsl::id.eq_any(ids))
            .load::<Code>(conn)
    }
}

#[derive(Queryable, Selectable, Insertable, Serialize, ToSchema)]
#[diesel(table_name = schema::idl)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Idl {
    pub id: String,
    pub content: String,
}

impl Idl {
    pub fn save(
        conn: &mut PgConnection,
        id: &str,
        content: String,
    ) -> Result<(), diesel::result::Error> {
        let idl = Idl {
            id: id.to_string(),
            content,
        };

        diesel::insert_into(schema::idl::table)
            .values(&idl)
            .returning(Idl::as_returning())
            .on_conflict_do_nothing()
            .execute(conn)?;

        Ok(())
    }

    pub fn get(conn: &mut PgConnection, id: &str) -> Option<Idl> {
        schema::idl::dsl::idl.find(id).first(conn).ok()
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::verification)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Clone, Debug)]
pub struct Verification {
    pub id: String,
    pub repo_link: String,
    pub code_id: String,
    pub project_name: Option<String>,
    pub manifest_path: Option<String>,
    pub base_path: Option<String>,
    pub build_idl: bool,
    pub version: String,
    pub status: VerificationStatus,
    pub network: Network,
    pub failed_reason: Option<String>,
    pub created_at: SystemTime,
}

impl Verification {
    pub fn save(conn: &mut PgConnection, verif: Verification) -> Self {
        diesel::insert_into(schema::verification::table)
            .values(&verif)
            .returning(Verification::as_returning())
            .get_result(conn)
            .expect("Error saving Verification")
    }

    pub fn get(conn: &mut PgConnection, id: &str) -> Option<Verification> {
        verif_dsl::verification.find(id).first(conn).ok()
    }

    pub fn is_verification_in_progress(
        conn: &mut PgConnection,
        code_id: &str,
        cur_verif_id: &str,
    ) -> bool {
        verif_dsl::verification
            .filter(verif_dsl::code_id.eq(code_id))
            .filter(verif_dsl::id.ne(cur_verif_id))
            .filter(verif_dsl::status.eq::<VerificationStatus>(VerificationStatus::InProgress))
            .first::<Verification>(conn)
            .is_ok()
    }

    pub fn update(
        conn: &mut PgConnection,
        id: &str,
        status: VerificationStatus,
        reason: Option<String>,
    ) -> Result<usize, anyhow::Error> {
        diesel::update(verif_dsl::verification.find(id))
            .set((
                verif_dsl::status.eq(status),
                verif_dsl::failed_reason.eq(reason),
            ))
            .execute(conn)
            .map_err(|e| anyhow::anyhow!("Failed to update verification {}. Error: {:?}", id, e))
    }

    pub fn get_pending(conn: &mut PgConnection, count: i64) -> Vec<Verification> {
        verif_dsl::verification
            .filter(verif_dsl::status.eq::<VerificationStatus>(VerificationStatus::Pending))
            .order_by(verif_dsl::created_at)
            .limit(count)
            .load::<Verification>(conn)
            .expect("Error loading pending verifications")
    }

    pub fn reset_in_progress(conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        diesel::update(
            verif_dsl::verification.filter(verif_dsl::status.eq(VerificationStatus::InProgress)),
        )
        .set(verif_dsl::status.eq(VerificationStatus::Pending))
        .execute(conn)
    }
}

#[derive(Debug, AsExpression, FromSqlRow, Serialize, Clone)]
#[diesel(sql_type = schema::sql_types::Verificationstatus)]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Verified,
    Failed,
}

impl ToSql<schema::sql_types::Verificationstatus, diesel::pg::Pg> for VerificationStatus {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        match &self {
            VerificationStatus::Pending => out.write_all(b"pending")?,
            VerificationStatus::InProgress => out.write_all(b"in_progress")?,
            VerificationStatus::Verified => out.write_all(b"verified")?,
            VerificationStatus::Failed => out.write_all(b"failed")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<schema::sql_types::Verificationstatus, diesel::pg::Pg> for VerificationStatus {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"pending" => Ok(VerificationStatus::Pending),
            b"in_progress" => Ok(VerificationStatus::InProgress),
            b"verified" => Ok(VerificationStatus::Verified),
            b"failed" => Ok(VerificationStatus::Failed),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<VerificationStatus> for String {
    fn from(status: VerificationStatus) -> String {
        match status {
            VerificationStatus::Pending => "pending".to_string(),
            VerificationStatus::InProgress => "in_progress".to_string(),
            VerificationStatus::Verified => "verified".to_string(),
            VerificationStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, Serialize, Clone)]
#[diesel(sql_type = schema::sql_types::Network)]
pub enum Network {
    VaraMainnet,
    VaraTestnet,
}

impl ToSql<schema::sql_types::Network, diesel::pg::Pg> for Network {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        match &self {
            Network::VaraMainnet => out.write_all(b"vara_mainnet")?,
            Network::VaraTestnet => out.write_all(b"vara_testnet")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<schema::sql_types::Network, diesel::pg::Pg> for Network {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"vara_mainnet" => Ok(Network::VaraMainnet),
            b"vara_testnet" => Ok(Network::VaraTestnet),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<Network> for String {
    fn from(status: Network) -> String {
        match status {
            Network::VaraMainnet => "vara_mainnet".to_string(),
            Network::VaraTestnet => "vara_testnet".to_string(),
        }
    }
}

impl TryFrom<String> for Network {
    type Error = anyhow::Error;

    fn try_from(status: String) -> Result<Network, Self::Error> {
        match status.as_str() {
            "vara_mainnet" => Ok(Network::VaraMainnet),
            "vara_testnet" => Ok(Network::VaraTestnet),
            _ => anyhow::bail!(
                "Unrecognized network name. Available options: vara_mainnet, vara_testnet"
            ),
        }
    }
}
