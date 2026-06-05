//! Database layer for the DAO service

use crate::error::DAOError;
use crate::types::*;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

/// Database connection and operations
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DAOError> {
        let pool = SqlitePool::connect(database_url).await?;
        
        let db = Self { pool };
        db.initialize_tables().await?;
        
        Ok(db)
    }

    /// Initialize database tables
    async fn initialize_tables(&self) -> Result<(), DAOError> {
        // Create proposals table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS proposals (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                proposal_type TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                proposer TEXT NOT NULL,
                proposer_did TEXT NOT NULL,
                status TEXT NOT NULL,
                execution_data TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create votes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS votes (
                id TEXT PRIMARY KEY,
                proposal_id TEXT NOT NULL,
                voter TEXT NOT NULL,
                voter_did TEXT NOT NULL,
                choice TEXT NOT NULL,
                weight INTEGER NOT NULL,
                reason TEXT,
                signature TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (proposal_id) REFERENCES proposals (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create execution_results table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS execution_results (
                proposal_id TEXT PRIMARY KEY,
                success BOOLEAN NOT NULL,
                output TEXT,
                error TEXT,
                gas_used INTEGER,
                tx_hash TEXT,
                executed_at TEXT NOT NULL,
                executor TEXT NOT NULL,
                FOREIGN KEY (proposal_id) REFERENCES proposals (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create members table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS members (
                address TEXT PRIMARY KEY,
                did TEXT NOT NULL UNIQUE,
                reputation INTEGER NOT NULL DEFAULT 0,
                stake INTEGER NOT NULL DEFAULT 0,
                voting_power INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'active',
                joined_at TEXT NOT NULL,
                last_activity TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create governance_params table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS governance_params (
                id INTEGER PRIMARY KEY,
                min_voting_period INTEGER NOT NULL,
                max_voting_period INTEGER NOT NULL,
                min_votes_required INTEGER NOT NULL,
                majority_threshold INTEGER NOT NULL,
                quorum_threshold INTEGER NOT NULL,
                execution_delay INTEGER NOT NULL,
                proposal_deposit INTEGER NOT NULL,
                voting_power_method TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Insert default governance parameters
        let default_params = GovernanceParams::default();
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO governance_params (
                id, min_voting_period, max_voting_period, min_votes_required,
                majority_threshold, quorum_threshold, execution_delay,
                proposal_deposit, voting_power_method, updated_at
            ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(default_params.min_voting_period as i64)
        .bind(default_params.max_voting_period as i64)
        .bind(default_params.min_votes_required as i64)
        .bind(default_params.majority_threshold as i64)
        .bind(default_params.quorum_threshold as i64)
        .bind(default_params.execution_delay as i64)
        .bind(default_params.proposal_deposit as i64)
        .bind(serde_json::to_string(&default_params.voting_power_method)?)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Create a proposal
    pub async fn create_proposal(&self, proposal: &Proposal) -> Result<(), DAOError> {
        let execution_data = proposal.execution_data.as_ref()
            .map(|data| serde_json::to_string(data))
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO proposals (
                id, title, description, proposal_type, start_time, end_time,
                proposer, proposer_did, status, execution_data, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(proposal.id.to_string())
        .bind(&proposal.title)
        .bind(&proposal.description)
        .bind(serde_json::to_string(&proposal.proposal_type)?)
        .bind(proposal.start_time.to_rfc3339())
        .bind(proposal.end_time.to_rfc3339())
        .bind(&proposal.proposer)
        .bind(&proposal.proposer_did)
        .bind(serde_json::to_string(&proposal.status)?)
        .bind(execution_data)
        .bind(proposal.created_at.to_rfc3339())
        .bind(proposal.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a proposal by ID
    pub async fn get_proposal(&self, id: &Uuid) -> Result<Option<Proposal>, DAOError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, proposal_type, start_time, end_time,
                   proposer, proposer_did, status, execution_data, created_at, updated_at
            FROM proposals WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let execution_data = if let Some(data) = row.get::<Option<String>, _>("execution_data") {
                Some(serde_json::from_str(&data)?)
            } else {
                None
            };

            let proposal = Proposal {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                title: row.get("title"),
                description: row.get("description"),
                proposal_type: serde_json::from_str(&row.get::<String, _>("proposal_type"))?,
                start_time: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))?.into(),
                end_time: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("end_time"))?.into(),
                proposer: row.get("proposer"),
                proposer_did: row.get("proposer_did"),
                status: serde_json::from_str(&row.get::<String, _>("status"))?,
                execution_data,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.into(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?.into(),
            };

            Ok(Some(proposal))
        } else {
            Ok(None)
        }
    }

    /// Update a proposal
    pub async fn update_proposal(&self, proposal: &Proposal) -> Result<(), DAOError> {
        let execution_data = proposal.execution_data.as_ref()
            .map(|data| serde_json::to_string(data))
            .transpose()?;

        sqlx::query(
            r#"
            UPDATE proposals SET
                title = ?, description = ?, proposal_type = ?, start_time = ?, end_time = ?,
                proposer = ?, proposer_did = ?, status = ?, execution_data = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&proposal.title)
        .bind(&proposal.description)
        .bind(serde_json::to_string(&proposal.proposal_type)?)
        .bind(proposal.start_time.to_rfc3339())
        .bind(proposal.end_time.to_rfc3339())
        .bind(&proposal.proposer)
        .bind(&proposal.proposer_did)
        .bind(serde_json::to_string(&proposal.status)?)
        .bind(execution_data)
        .bind(proposal.updated_at.to_rfc3339())
        .bind(proposal.id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a vote
    pub async fn create_vote(&self, vote: &Vote) -> Result<(), DAOError> {
        sqlx::query(
            r#"
            INSERT INTO votes (
                id, proposal_id, voter, voter_did, choice, weight, reason, signature, timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(vote.id.to_string())
        .bind(vote.proposal_id.to_string())
        .bind(&vote.voter)
        .bind(&vote.voter_did)
        .bind(serde_json::to_string(&vote.choice)?)
        .bind(vote.weight as i64)
        .bind(&vote.reason)
        .bind(&vote.signature)
        .bind(vote.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get votes for a proposal
    pub async fn get_votes_for_proposal(&self, proposal_id: &Uuid) -> Result<Vec<Vote>, DAOError> {
        let rows = sqlx::query(
            r#"
            SELECT id, proposal_id, voter, voter_did, choice, weight, reason, signature, timestamp
            FROM votes WHERE proposal_id = ?
            "#,
        )
        .bind(proposal_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut votes = Vec::new();
        for row in rows {
            let vote = Vote {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                proposal_id: Uuid::parse_str(&row.get::<String, _>("proposal_id"))?,
                voter: row.get("voter"),
                voter_did: row.get("voter_did"),
                choice: serde_json::from_str(&row.get::<String, _>("choice"))?,
                weight: row.get::<i64, _>("weight") as u64,
                reason: row.get("reason"),
                signature: row.get("signature"),
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))?.into(),
            };
            votes.push(vote);
        }

        Ok(votes)
    }

    /// Get governance parameters
    pub async fn get_governance_params(&self) -> Result<GovernanceParams, DAOError> {
        let row = sqlx::query(
            r#"
            SELECT min_voting_period, max_voting_period, min_votes_required,
                   majority_threshold, quorum_threshold, execution_delay,
                   proposal_deposit, voting_power_method
            FROM governance_params WHERE id = 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let params = GovernanceParams {
            min_voting_period: row.get::<i64, _>("min_voting_period") as u64,
            max_voting_period: row.get::<i64, _>("max_voting_period") as u64,
            min_votes_required: row.get::<i64, _>("min_votes_required") as u64,
            majority_threshold: row.get::<i64, _>("majority_threshold") as u8,
            quorum_threshold: row.get::<i64, _>("quorum_threshold") as u8,
            execution_delay: row.get::<i64, _>("execution_delay") as u64,
            proposal_deposit: row.get::<i64, _>("proposal_deposit") as u64,
            voting_power_method: serde_json::from_str(&row.get::<String, _>("voting_power_method"))?,
        };

        Ok(params)
    }

    /// Update governance parameters
    pub async fn update_governance_params(&self, params: &GovernanceParams) -> Result<(), DAOError> {
        sqlx::query(
            r#"
            UPDATE governance_params SET
                min_voting_period = ?, max_voting_period = ?, min_votes_required = ?,
                majority_threshold = ?, quorum_threshold = ?, execution_delay = ?,
                proposal_deposit = ?, voting_power_method = ?, updated_at = ?
            WHERE id = 1
            "#,
        )
        .bind(params.min_voting_period as i64)
        .bind(params.max_voting_period as i64)
        .bind(params.min_votes_required as i64)
        .bind(params.majority_threshold as i64)
        .bind(params.quorum_threshold as i64)
        .bind(params.execution_delay as i64)
        .bind(params.proposal_deposit as i64)
        .bind(serde_json::to_string(&params.voting_power_method)?)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
