use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::{certificate::VotePlanId, vote::Options};
use chain_vote::ElectionPublicKey;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, fmt, str};
pub use wallet_core::{Choice, Value};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Fund {
    pub id: i32,
    #[serde(alias = "fundName")]
    pub fund_name: String,
    #[serde(alias = "fundGoal")]
    pub fund_goal: String,
    pub registration_snapshot_time: String,
    pub voting_power_threshold: u64,
    #[serde(alias = "fundStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub fund_start_time: i64,
    #[serde(alias = "fundEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub fund_end_time: i64,
    #[serde(alias = "nextFundStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub next_fund_start_time: i64,
    #[serde(alias = "chainVotePlans")]
    pub chain_vote_plans: Vec<Voteplan>,
    #[serde(alias = "chainVotePlans")]
    pub challenges: Vec<Challenge>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Voteplan {
    pub id: i32,
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainVoteStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEnd")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_committee_end_time: i64,
    #[serde(alias = "chainVoteplanPayload")]
    pub chain_voteplan_payload: String,
    pub chain_vote_encryption_key: String,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]

pub struct Category {
    #[serde(alias = "categoryId")]
    pub category_id: String,
    #[serde(alias = "categoryName")]
    pub category_name: String,
    #[serde(alias = "categoryDescription")]
    pub category_description: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Proposer {
    #[serde(alias = "proposerName")]
    pub proposer_name: String,
    #[serde(alias = "proposerEmail")]
    pub proposer_email: String,
    #[serde(alias = "proposerUrl")]
    pub proposer_url: String,
    pub proposer_relevant_experience: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Proposal {
    #[serde(alias = "internalId")]
    pub internal_id: i64,
    #[serde(alias = "proposalId")]
    pub proposal_id: String,
    //  #[serde(alias = "category")]
    pub proposal_category: Category,
    #[serde(alias = "proposalTitle")]
    pub proposal_title: String,
    #[serde(alias = "proposalSummary")]
    pub proposal_summary: String,
    #[serde(alias = "proposalProblem")]
    pub proposal_problem: Option<String>,
    #[serde(alias = "proposalSolution")]
    pub proposal_solution: Option<String>,
    #[serde(alias = "proposalPublicKey")]
    pub proposal_public_key: String,
    #[serde(alias = "proposalFunds")]
    pub proposal_funds: i64,
    #[serde(alias = "proposalUrl")]
    pub proposal_url: String,
    #[serde(alias = "proposalFilesUrl")]
    pub proposal_files_url: String,
    pub proposal_impact_score: u32,
    #[serde(alias = "challenge_id")]
    pub challenge_id: u32,
    pub proposer: Proposer,
    #[serde(alias = "chainProposalId")]
    #[serde(serialize_with = "crate::utils::serde::serialize_bin_as_str")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_string_as_bytes")]
    pub chain_proposal_id: Vec<u8>,
    #[serde(alias = "chainProposalIndex")]
    pub chain_proposal_index: i64,
    #[serde(alias = "chainVoteOptions")]
    pub chain_vote_options: VoteOptions,
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainVoteplanPayload")]
    pub chain_voteplan_payload: String,
    #[serde(alias = "chainVoteEncryptionKey")]
    pub chain_vote_encryption_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdvisorReview {
    pub id: i32,
    pub proposal_id: i32,
    pub rating_given: i32,
    pub assessor: String,
    pub note: String,
    pub tag: ReviewTag,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReviewTag {
    Alignment,
    Verifiability,
    Feasibility,
    Impact,
    Auditability,
}

impl ToString for ReviewTag {
    fn to_string(&self) -> String {
        match self {
            ReviewTag::Alignment => "Alignment",
            ReviewTag::Verifiability => "Verifiability",
            ReviewTag::Feasibility => "Feasibility",
            ReviewTag::Impact => "Impact",
            ReviewTag::Auditability => "Auditability",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: i32,
    #[serde(alias = "challengeType")]
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "challengeUrl")]
    pub challenge_url: String,
    #[serde(alias = "proposers_rewards")]
    pub proposers_rewards: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChallengeType {
    Simple,
    CommunityChoice,
}

impl std::fmt::Display for ChallengeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // should be implemented and safe to unwrap here
        let repr = serde_json::to_string(&self).unwrap();
        write!(f, "{}", repr.trim_matches('"'))
    }
}

impl Proposal {
    pub fn chain_proposal_id_as_str(&self) -> String {
        str::from_utf8(&self.chain_proposal_id).unwrap().to_string()
    }

    pub fn get_option_text(&self, choice: u8) -> String {
        self.chain_vote_options
            .0
            .iter()
            .find(|(_, y)| **y == choice)
            .map(|(x, _)| x.to_string())
            .unwrap()
    }
}

#[allow(clippy::from_over_into)]
impl Into<wallet_core::Proposal> for Proposal {
    fn into(self) -> wallet_core::Proposal {
        let chain_proposal_index = self.chain_proposal_index as u8;
        let bytes = hex::decode(self.chain_voteplan_id).unwrap();
        let mut vote_plan_id = [0; 32];
        let bytes = &bytes[..vote_plan_id.len()]; // panics if not enough data
        vote_plan_id.copy_from_slice(bytes);

        if self.chain_voteplan_payload == "public" {
            return wallet_core::Proposal::new_public(
                VotePlanId::try_from(vote_plan_id).unwrap(),
                chain_proposal_index,
                Options::new_length(self.chain_vote_options.0.len() as u8).unwrap(),
            );
        }
        wallet_core::Proposal::new_private(
            VotePlanId::try_from(vote_plan_id).unwrap(),
            chain_proposal_index,
            Options::new_length(self.chain_vote_options.0.len() as u8).unwrap(),
            ElectionPublicKey::try_from_bech32_str(&self.chain_vote_encryption_key).unwrap(),
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServiceVersion {
    pub service_version: String,
}

pub struct SimpleVoteStatus {
    pub chain_proposal_id: String,
    pub proposal_title: String,
    pub choice: String,
}

impl fmt::Display for SimpleVoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# {}, '{}' -> Choice:  {}",
            self.chain_proposal_id, self.proposal_title, self.choice
        )
    }
}

pub type VoteOptionsMap = HashMap<String, u8>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct VoteOptions(pub VoteOptionsMap);

impl VoteOptions {
    pub fn as_csv_string(&self) -> String {
        self.0
            .iter()
            .sorted_by_key(|(_, &i)| i)
            .map(|(v, _)| v)
            .join(",")
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct VitVersion {
    service_version: String,
}

impl VitVersion {
    pub fn new(service_version: String) -> Self {
        Self { service_version }
    }

    pub fn version(&self) -> String {
        self.service_version.clone()
    }
}

impl Default for VitVersion {
    fn default() -> Self {
        Self {
            service_version: "2.0".to_string(),
        }
    }
}
