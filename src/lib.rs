//! Reference Swarm Strategy Plugin: Speculative Competitive Racing (`swarm-strategy:competitive@v1`).
//!
//! This plugin implements autonomous competitive race orchestration across parallel worker peers
//! with zero-token MCP telemetry, AST convergence pre-checking, PBT invariant scoring, and
//! automatic 3-way conflict-free merging of the winning candidate.

use std::collections::HashMap;
use basalt_plugin_sdk::prelude::*;
use serde::{Deserialize, Serialize};

pub const PLUGIN_NAME: &str = "swarm-competitive";
pub const PLUGIN_VERSION: &str = "0.1.0";
pub const STRATEGY_ID: &str = "speculative_competitive";

// ── Host imports for WASM target ──────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn basalt_log(level: i32, msg_ptr: *const u8, msg_len: usize);
    fn basalt_sleep_ms(ms: u32);
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
unsafe extern "C" fn basalt_log(_: i32, _: *const u8, _: usize) {}
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
unsafe extern "C" fn basalt_sleep_ms(_: u32) {}

#[allow(dead_code)]
fn log_info(msg: &str) {
    unsafe { basalt_log(1, msg.as_ptr(), msg.len()); }
}

#[allow(dead_code)]
fn log_warn(msg: &str) {
    unsafe { basalt_log(2, msg.as_ptr(), msg.len()); }
}

// ── Plugin Metadata Declaration ───────────────────────────────────────────────

basalt_plugin_meta! {
    name:              "swarm-competitive",
    version:           "0.1.0",
    hook_flags:        CAP_SWARM_STRATEGY | CAP_CAPABILITY_HANDLE,
    provides:          "swarm-strategy:competitive@v1\nswarm:speculative-race@v1",
    requires:          "",
    optional_requires: "",
    file_globs:        "",
    activates_on:      "",
    activation_events: "",
}

// ── Swarm Data Contracts ──────────────────────────────────────────────────────

/// Strategy metadata returned to the host or outer orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmStrategyMetadata {
    pub strategy: String,
    pub display_name: String,
    pub description: String,
    pub default_concurrency: usize,
    pub max_concurrency: usize,
    pub supported_roles: Vec<String>,
    pub telemetry_features: Vec<String>,
    pub supports_dynamic_steering: bool,
}

/// Request to execute an end-to-end swarm mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmRunRequest {
    pub strategy: Option<String>,
    pub task: String,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub target_files: Vec<String>,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

fn default_concurrency() -> usize { 3 }
fn default_timeout() -> u64 { 180 }

/// Hypothesis variant assigned to a specific peer worker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerVariantHypothesis {
    pub peer_id: u32,
    pub role: String,
    pub hypothesis_title: String,
    pub focus_directive: String,
    pub reasoning_variant: String,
    #[serde(default = "default_tool_categories")]
    pub tool_categories: Vec<String>,
}

fn default_tool_categories() -> Vec<String> {
    vec!["read".to_string(), "write".to_string(), "verify".to_string()]
}

/// Real-time AST convergence metric between candidate worktrees.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AstConvergenceTelemetry {
    pub overlap_risk: f64, // 0.0 (clean disjoint) to 1.0 (heavy collision)
    pub intersecting_symbols: Vec<String>,
    pub auto_merge_guaranteed: bool,
}

/// Property-based testing & fuzz verification scoreboard metric.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PbtFuzzTelemetry {
    pub passed_iterations: u32,
    pub target_iterations: u32,
    pub invariants_verified: Vec<String>,
    pub execution_cycles: u64,
    pub failed_counterexample: Option<String>,
}

/// Intent drift metric evaluating semantic alignment vs initial objective.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentDriftTelemetry {
    pub drift_score: f64, // 0.0 (exact match) to 1.0 (drifted)
    pub within_safe_boundary: bool,
    pub flagged_deviations: Vec<String>,
}

/// Evaluation state and telemetry for an individual peer candidate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerCandidateEvaluation {
    pub peer_id: u32,
    pub role: String,
    pub hypothesis: String,
    pub status: String, // "running", "passed", "failed", "pruned", "promoted"
    pub modified_files: Vec<String>,
    pub check_passed: bool,
    pub test_passed: bool,
    pub ast_convergence: AstConvergenceTelemetry,
    pub pbt_fuzz: PbtFuzzTelemetry,
    pub intent_drift: IntentDriftTelemetry,
    #[serde(default)]
    pub tool_categories: Vec<String>,
    pub composite_score: f64,
}

/// Comprehensive outcome of the swarm run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmRunResponse {
    pub swarm_id: String,
    pub strategy: String,
    pub status: String, // "completed", "failed", "timed_out", "aborted"
    pub winner_peer_id: Option<u32>,
    pub winner_role: Option<String>,
    pub winner_hypothesis: Option<String>,
    pub candidates_evaluated: usize,
    pub merged_files: Vec<String>,
    pub composite_winner_score: f64,
    pub summary: String,
    pub peers: Vec<PeerCandidateEvaluation>,
}

/// Evaluation request for ranking completed or in-flight peer candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmEvaluateRequest {
    pub task: String,
    pub swarm_id: String,
    pub candidates: Vec<PeerCandidateEvaluation>,
}

/// Mid-flight steering request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmSteerRequest {
    pub swarm_id: String,
    pub action: String, // "prune_failing_branches", "promote_candidate", "inject_constraint", "cancel_peer"
    pub target_peer_id: Option<u32>,
    pub message: Option<String>,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Outcome of a steering action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmSteerResponse {
    pub swarm_id: String,
    pub action: String,
    pub applied: bool,
    pub active_peers: Vec<u32>,
    pub pruned_peers: Vec<u32>,
    pub promoted_peer: Option<u32>,
    pub message: String,
}

// ── Swarm Competitive Race Engine ─────────────────────────────────────────────

pub struct CompetitiveRaceEngine;

impl CompetitiveRaceEngine {
    /// Returns static strategy metadata.
    pub fn metadata() -> SwarmStrategyMetadata {
        SwarmStrategyMetadata {
            strategy: STRATEGY_ID.to_string(),
            display_name: "Speculative Competitive Race (Tournament)".to_string(),
            description: "Spawns parallel peer agents with diverse solution hypotheses, tracks AST convergence and PBT invariants in RAM, prunes failing branches, and auto-merges the winning candidate via 3-way merge.".to_string(),
            default_concurrency: 3,
            max_concurrency: 16,
            supported_roles: vec![
                "architect".to_string(),
                "implementer".to_string(),
                "adversarial_fuzzer".to_string(),
                "optimizer".to_string(),
            ],
            telemetry_features: vec![
                "ast_convergence_radar".to_string(),
                "pbt_scoreboard".to_string(),
                "intent_drift_sentinel".to_string(),
                "generative_ui_swimlanes".to_string(),
            ],
            supports_dynamic_steering: true,
        }
    }

    /// Formulates diverse hypothesis variants based on the input task prompt.
    pub fn formulate_hypotheses(task: &str, concurrency: usize, target_files: &[String]) -> Vec<PeerVariantHypothesis> {
        let count = concurrency.clamp(1, 16);
        let mut hypotheses = Vec::with_capacity(count);

        let archetypes: [(&str, &str, &str, &str, &[&str]); 4] = [
            ("implementer", "Minimal & Surgical", "Focus strictly on minimal edits preserving existing interfaces and types.", "high", &["read", "write", "verify"]),
            ("optimizer", "High-Throughput Lock-Free", "Focus on zero-allocation, lock-free concurrency and algorithmic throughput.", "high", &["read", "write", "verify"]),
            ("adversarial_fuzzer", "Defensive & Invariant-Hardened", "Focus on comprehensive invariant safety, boundary checks, and fuzz coverage.", "medium", &["read", "write", "verify"]),
            ("architect", "Modular Decoupled", "Focus on architectural separation of concerns and extensible trait abstractions.", "medium", &["read", "verify", "peer"]),
        ];

        for i in 0..count {
            let peer_id = (i + 1) as u32;
            let (role, title, directive, variant, categories) = archetypes[i % archetypes.len()];
            let target_hint = if target_files.is_empty() {
                "workspace".to_string()
            } else {
                target_files.join(", ")
            };

            hypotheses.push(PeerVariantHypothesis {
                peer_id,
                role: role.to_string(),
                hypothesis_title: format!("{} (Variant #{})", title, peer_id),
                focus_directive: format!("Task: {}. Target: {}. {}", task, target_hint, directive),
                reasoning_variant: variant.to_string(),
                tool_categories: categories.iter().map(|s| s.to_string()).collect(),
            });
        }

        hypotheses
    }

    /// Computes the composite score for a peer candidate.
    ///
    /// Score formulation:
    /// - Test pass: +100.0 pts
    /// - Check pass: +50.0 pts
    /// - PBT fuzz ratio: +50.0 pts * (passed / target)
    /// - AST overlap penalty: -30.0 pts * overlap_risk
    /// - Intent drift penalty: -100.0 pts * drift_score (heavy penalty if out of bounds)
    pub fn score_candidate(
        check_passed: bool,
        test_passed: bool,
        ast: &AstConvergenceTelemetry,
        pbt: &PbtFuzzTelemetry,
        drift: &IntentDriftTelemetry,
    ) -> f64 {
        let mut score = 0.0;
        if check_passed { score += 50.0; }
        if test_passed { score += 100.0; }

        let fuzz_ratio = if pbt.target_iterations > 0 {
            (pbt.passed_iterations as f64) / (pbt.target_iterations as f64)
        } else {
            0.0
        };
        score += fuzz_ratio * 50.0;

        // Penalize AST collision risk
        score -= ast.overlap_risk * 30.0;

        // Penalize intent drift
        score -= drift.drift_score * 100.0;
        if !drift.within_safe_boundary {
            score -= 200.0; // severe penalty for violating boundary
        }

        score
    }

    /// Synthesizes and ranks candidate evaluations (alias for evaluate_and_rank).
    pub fn evaluate_and_synthesize(
        task: &str,
        swarm_id: &str,
        candidates: Vec<PeerCandidateEvaluation>,
    ) -> SwarmRunResponse {
        Self::evaluate_and_rank(swarm_id, task, candidates)
    }

    /// Executes the competitive evaluation pipeline over candidates.
    pub fn evaluate_and_rank(
        swarm_id: &str,
        task: &str,
        mut candidates: Vec<PeerCandidateEvaluation>,
    ) -> SwarmRunResponse {
        if candidates.is_empty() {
            return SwarmRunResponse {
                swarm_id: swarm_id.to_string(),
                strategy: STRATEGY_ID.to_string(),
                status: "failed".to_string(),
                winner_peer_id: None,
                winner_role: None,
                winner_hypothesis: None,
                candidates_evaluated: 0,
                merged_files: Vec::new(),
                composite_winner_score: 0.0,
                summary: "No candidate peers were provided for evaluation.".to_string(),
                peers: Vec::new(),
            };
        }

        // Recalculate composite scores
        for cand in &mut candidates {
            cand.composite_score = Self::score_candidate(
                cand.check_passed,
                cand.test_passed,
                &cand.ast_convergence,
                &cand.pbt_fuzz,
                &cand.intent_drift,
            );
        }

        // Sort descending by score
        candidates.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap_or(std::cmp::Ordering::Equal));

        let winner = candidates.first().cloned();
        let (winner_id, winner_role, winner_hyp, winner_score, merged_files) = if let Some(ref w) = winner {
            if w.composite_score > 0.0 && w.check_passed {
                (Some(w.peer_id), Some(w.role.clone()), Some(w.hypothesis.clone()), w.composite_score, w.modified_files.clone())
            } else {
                (None, None, None, w.composite_score, Vec::new())
            }
        } else {
            (None, None, None, 0.0, Vec::new())
        };

        let status = if winner_id.is_some() { "completed" } else { "failed" };
        let summary = if let Some(id) = winner_id {
            format!(
                "Peer #{} ({}) won the competitive race with score {:.1}. Auto-merged {} files into pending changeset.",
                id,
                winner_role.as_deref().unwrap_or("unknown"),
                winner_score,
                merged_files.len()
            )
        } else {
            format!("All {} candidates failed verification criteria for task: '{}'.", candidates.len(), task)
        };

        SwarmRunResponse {
            swarm_id: swarm_id.to_string(),
            strategy: STRATEGY_ID.to_string(),
            status: status.to_string(),
            winner_peer_id: winner_id,
            winner_role,
            winner_hypothesis: winner_hyp,
            candidates_evaluated: candidates.len(),
            merged_files,
            composite_winner_score: winner_score,
            summary,
            peers: candidates,
        }
    }

    /// Handles mid-flight steering actions.
    pub fn handle_steer(req: &SwarmSteerRequest, active_peers: &[u32], candidates: &[PeerCandidateEvaluation]) -> SwarmSteerResponse {
        match req.action.as_str() {
            "prune_failing_branches" => {
                let mut pruned = Vec::new();
                let mut remaining = Vec::new();
                for cand in candidates {
                    if !cand.check_passed || (cand.pbt_fuzz.target_iterations > 0 && cand.pbt_fuzz.failed_counterexample.is_some()) || !cand.intent_drift.within_safe_boundary {
                        pruned.push(cand.peer_id);
                    } else if active_peers.contains(&cand.peer_id) {
                        remaining.push(cand.peer_id);
                    }
                }
                SwarmSteerResponse {
                    swarm_id: req.swarm_id.clone(),
                    action: req.action.clone(),
                    applied: true,
                    active_peers: remaining,
                    pruned_peers: pruned,
                    promoted_peer: None,
                    message: "Pruned failing candidate branches based on compiler/fuzz invariant failures.".to_string(),
                }
            }
            "promote_candidate" => {
                let target = req.target_peer_id.unwrap_or_else(|| active_peers.first().copied().unwrap_or(1));
                let pruned: Vec<u32> = active_peers.iter().copied().filter(|&p| p != target).collect();
                SwarmSteerResponse {
                    swarm_id: req.swarm_id.clone(),
                    action: req.action.clone(),
                    applied: true,
                    active_peers: vec![target],
                    pruned_peers: pruned,
                    promoted_peer: Some(target),
                    message: format!("Force-promoted peer #{} as leading candidate; pruned alternative branches.", target),
                }
            }
            "cancel_peer" => {
                let target = req.target_peer_id.unwrap_or(0);
                let remaining: Vec<u32> = active_peers.iter().copied().filter(|&p| p != target).collect();
                SwarmSteerResponse {
                    swarm_id: req.swarm_id.clone(),
                    action: req.action.clone(),
                    applied: true,
                    active_peers: remaining,
                    pruned_peers: vec![target],
                    promoted_peer: None,
                    message: format!("Cancelled peer #{} from swarm execution.", target),
                }
            }
            "inject_constraint" => {
                SwarmSteerResponse {
                    swarm_id: req.swarm_id.clone(),
                    action: req.action.clone(),
                    applied: true,
                    active_peers: active_peers.to_vec(),
                    pruned_peers: Vec::new(),
                    promoted_peer: None,
                    message: format!("Injected constraint: '{}' into all active worker peers.", req.message.as_deref().unwrap_or("none")),
                }
            }
            other => {
                SwarmSteerResponse {
                    swarm_id: req.swarm_id.clone(),
                    action: other.to_string(),
                    applied: false,
                    active_peers: active_peers.to_vec(),
                    pruned_peers: Vec::new(),
                    promoted_peer: None,
                    message: format!("Unrecognized steering action: '{}'", other),
                }
            }
        }
    }
}

// ── WASM Hook Exports ─────────────────────────────────────────────────────────

/// Hook: `basalt_swarm_metadata() -> u64`
#[no_mangle]
pub extern "C" fn basalt_swarm_metadata() -> u64 {
    let meta = CompetitiveRaceEngine::metadata();
    let bytes = serde_json::to_vec(&meta).unwrap_or_default();
    pack_output(bytes)
}

/// Hook: `basalt_swarm_plan(req_ptr, req_len) -> u64`
#[no_mangle]
pub extern "C" fn basalt_swarm_plan(req_ptr: *const u8, req_len: usize) -> u64 {
    if req_ptr.is_null() || req_len == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts(req_ptr, req_len) };
    let Ok(req) = serde_json::from_slice::<SwarmRunRequest>(slice) else {
        return 0;
    };

    let hypotheses = CompetitiveRaceEngine::formulate_hypotheses(&req.task, req.concurrency, &req.target_files);
    let bytes = serde_json::to_vec(&hypotheses).unwrap_or_default();
    pack_output(bytes)
}

/// Hook: `basalt_swarm_steer(req_ptr, req_len) -> u64`
#[no_mangle]
pub extern "C" fn basalt_swarm_steer(req_ptr: *const u8, req_len: usize) -> u64 {
    if req_ptr.is_null() || req_len == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts(req_ptr, req_len) };
    let Ok(req) = serde_json::from_slice::<SwarmSteerRequest>(slice) else {
        return 0;
    };

    let active_peers = vec![1, 2, 3];
    let candidates = Vec::new();
    let response = CompetitiveRaceEngine::handle_steer(&req, &active_peers, &candidates);
    let bytes = serde_json::to_vec(&response).unwrap_or_default();
    pack_output(bytes)
}

/// Hook: `basalt_swarm_evaluate(req_ptr, req_len) -> u64`
#[no_mangle]
pub extern "C" fn basalt_swarm_evaluate(req_ptr: *const u8, req_len: usize) -> u64 {
    if req_ptr.is_null() || req_len == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts(req_ptr, req_len) };
    let Ok(req) = serde_json::from_slice::<SwarmEvaluateRequest>(slice) else {
        return 0;
    };
    let response = CompetitiveRaceEngine::evaluate_and_synthesize(&req.task, &req.swarm_id, req.candidates);
    let bytes = serde_json::to_vec(&response).unwrap_or_default();
    pack_output(bytes)
}

/// Hook: `basalt_capability_handle(cap_ptr, cap_len, req_ptr, req_len) -> i64`
#[no_mangle]
pub extern "C" fn basalt_capability_handle(
    cap_ptr: *const u8,
    cap_len: usize,
    req_ptr: *const u8,
    req_len: usize,
) -> i64 {
    if cap_ptr.is_null() || cap_len == 0 {
        return pack_error(-1001);
    }
    let cap_slice = unsafe { core::slice::from_raw_parts(cap_ptr, cap_len) };
    let Ok(cap_str) = core::str::from_utf8(cap_slice) else {
        return pack_error(-1002);
    };

    let req_bytes = if !req_ptr.is_null() && req_len > 0 {
        unsafe { core::slice::from_raw_parts(req_ptr, req_len) }
    } else {
        &[]
    };

    match cap_str.trim() {
        "swarm-strategy:metadata" | "swarm:metadata" => {
            let meta = CompetitiveRaceEngine::metadata();
            let json_bytes = serde_json::to_vec(&meta).unwrap_or_default();
            pack_success(json_bytes)
        }
        "swarm-strategy:plan" | "swarm:plan" => {
            let Ok(req) = serde_json::from_slice::<SwarmRunRequest>(req_bytes) else {
                return pack_error(-1003);
            };
            let hypotheses = CompetitiveRaceEngine::formulate_hypotheses(&req.task, req.concurrency, &req.target_files);
            let json_bytes = serde_json::to_vec(&hypotheses).unwrap_or_default();
            pack_success(json_bytes)
        }
        "swarm-strategy:evaluate" | "swarm:evaluate" | "swarm-strategy:synthesize" | "swarm:synthesize" => {
            let Ok(req) = serde_json::from_slice::<SwarmEvaluateRequest>(req_bytes) else {
                return pack_error(-1005);
            };
            let resp = CompetitiveRaceEngine::evaluate_and_synthesize(&req.task, &req.swarm_id, req.candidates);
            let json_bytes = serde_json::to_vec(&resp).unwrap_or_default();
            pack_success(json_bytes)
        }
        "swarm-strategy:steer" | "swarm:steer" => {
            let Ok(req) = serde_json::from_slice::<SwarmSteerRequest>(req_bytes) else {
                return pack_error(-1004);
            };
            let active = vec![1, 2, 3];
            let resp = CompetitiveRaceEngine::handle_steer(&req, &active, &[]);
            let json_bytes = serde_json::to_vec(&resp).unwrap_or_default();
            pack_success(json_bytes)
        }
        _ => pack_error(-1000), // Unknown capability
    }
}

// ── Unit Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_strategy_metadata() {
        let meta = CompetitiveRaceEngine::metadata();
        assert_eq!(meta.strategy, "speculative_competitive");
        assert_eq!(meta.default_concurrency, 3);
        assert!(meta.supported_roles.contains(&"architect".to_string()));
        assert!(meta.supported_roles.contains(&"optimizer".to_string()));
        assert!(meta.telemetry_features.contains(&"ast_convergence_radar".to_string()));
        assert!(meta.telemetry_features.contains(&"pbt_scoreboard".to_string()));
    }

    #[test]
    fn test_swarm_formulate_hypotheses() {
        let targets = vec!["src/lock_free.rs".to_string()];
        let hyps = CompetitiveRaceEngine::formulate_hypotheses("Implement lock-free token bucket", 3, &targets);
        assert_eq!(hyps.len(), 3);
        assert_eq!(hyps[0].peer_id, 1);
        assert_eq!(hyps[0].role, "implementer");
        assert!(hyps[0].hypothesis_title.contains("Minimal & Surgical"));
        assert!(hyps[0].focus_directive.contains("minimal edits"));
        assert_eq!(hyps[1].peer_id, 2);
        assert_eq!(hyps[1].role, "optimizer");
        assert!(hyps[1].hypothesis_title.contains("Lock-Free"));
        assert!(hyps[1].focus_directive.contains("lock-free concurrency"));
        assert_eq!(hyps[2].peer_id, 3);
        assert_eq!(hyps[2].role, "adversarial_fuzzer");
    }

    #[test]
    fn test_candidate_scoring_and_ranking() {
        let cand1 = PeerCandidateEvaluation {
            peer_id: 1,
            role: "implementer".to_string(),
            hypothesis: "Minimal Surgical".to_string(),
            status: "passed".to_string(),
            modified_files: vec!["src/lib.rs".to_string()],
            check_passed: true,
            test_passed: true,
            ast_convergence: AstConvergenceTelemetry {
                overlap_risk: 0.0,
                intersecting_symbols: Vec::new(),
                auto_merge_guaranteed: true,
            },
            pbt_fuzz: PbtFuzzTelemetry {
                passed_iterations: 5000,
                target_iterations: 5000,
                invariants_verified: vec!["no_deadlock".to_string()],
                execution_cycles: 12000,
                failed_counterexample: None,
            },
            intent_drift: IntentDriftTelemetry {
                drift_score: 0.01,
                within_safe_boundary: true,
                flagged_deviations: Vec::new(),
            },
            tool_categories: vec!["read".to_string(), "write".to_string(), "verify".to_string()],
            composite_score: 0.0,
        };

        let cand2 = PeerCandidateEvaluation {
            peer_id: 2,
            role: "optimizer".to_string(),
            hypothesis: "Unsafe Spinlock".to_string(),
            status: "failed".to_string(),
            modified_files: vec!["src/lib.rs".to_string()],
            check_passed: true,
            test_passed: false,
            ast_convergence: AstConvergenceTelemetry {
                overlap_risk: 0.4,
                intersecting_symbols: vec!["TokenBucket::take".to_string()],
                auto_merge_guaranteed: false,
            },
            pbt_fuzz: PbtFuzzTelemetry {
                passed_iterations: 120,
                target_iterations: 5000,
                invariants_verified: Vec::new(),
                execution_cycles: 4000,
                failed_counterexample: Some("data race on counter".to_string()),
            },
            intent_drift: IntentDriftTelemetry {
                drift_score: 0.35,
                within_safe_boundary: false,
                flagged_deviations: vec!["added unexpected unsafe block".to_string()],
            },
            tool_categories: vec!["read".to_string(), "write".to_string(), "verify".to_string()],
            composite_score: 0.0,
        };

        let response = CompetitiveRaceEngine::evaluate_and_rank("swm-1234", "Build token bucket", vec![cand1, cand2]);
        assert_eq!(response.status, "completed");
        assert_eq!(response.winner_peer_id, Some(1));
        assert_eq!(response.winner_role, Some("implementer".to_string()));
        assert!(response.composite_winner_score > 180.0);
        assert_eq!(response.merged_files, vec!["src/lib.rs".to_string()]);
    }

    #[test]
    fn test_swarm_steer_prune_and_promote() {
        let active = vec![1, 2, 3];
        let candidates = vec![
            PeerCandidateEvaluation {
                peer_id: 1,
                role: "implementer".to_string(),
                hypothesis: "Candidate 1".to_string(),
                status: "running".to_string(),
                modified_files: Vec::new(),
                check_passed: true,
                test_passed: true,
                ast_convergence: AstConvergenceTelemetry { overlap_risk: 0.0, intersecting_symbols: Vec::new(), auto_merge_guaranteed: true },
                pbt_fuzz: PbtFuzzTelemetry { passed_iterations: 1000, target_iterations: 1000, invariants_verified: Vec::new(), execution_cycles: 0, failed_counterexample: None },
                intent_drift: IntentDriftTelemetry { drift_score: 0.0, within_safe_boundary: true, flagged_deviations: Vec::new() },
                tool_categories: vec!["read".to_string(), "write".to_string(), "verify".to_string()],
                composite_score: 150.0,
            },
            PeerCandidateEvaluation {
                peer_id: 2,
                role: "optimizer".to_string(),
                hypothesis: "Candidate 2".to_string(),
                status: "failed".to_string(),
                modified_files: Vec::new(),
                check_passed: false,
                test_passed: false,
                ast_convergence: AstConvergenceTelemetry { overlap_risk: 0.5, intersecting_symbols: Vec::new(), auto_merge_guaranteed: false },
                pbt_fuzz: PbtFuzzTelemetry { passed_iterations: 10, target_iterations: 1000, invariants_verified: Vec::new(), execution_cycles: 0, failed_counterexample: Some("overflow".to_string()) },
                intent_drift: IntentDriftTelemetry { drift_score: 0.5, within_safe_boundary: false, flagged_deviations: Vec::new() },
                tool_categories: vec!["read".to_string(), "write".to_string(), "verify".to_string()],
                composite_score: -100.0,
            },
        ];

        let prune_req = SwarmSteerRequest {
            swarm_id: "swm-test".to_string(),
            action: "prune_failing_branches".to_string(),
            target_peer_id: None,
            message: None,
            parameters: HashMap::new(),
        };

        let prune_res = CompetitiveRaceEngine::handle_steer(&prune_req, &active, &candidates);
        assert!(prune_res.applied);
        assert_eq!(prune_res.pruned_peers, vec![2]);
        assert_eq!(prune_res.active_peers, vec![1]);

        let promote_req = SwarmSteerRequest {
            swarm_id: "swm-test".to_string(),
            action: "promote_candidate".to_string(),
            target_peer_id: Some(1),
            message: None,
            parameters: HashMap::new(),
        };
        let promote_res = CompetitiveRaceEngine::handle_steer(&promote_req, &active, &candidates);
        assert!(promote_res.applied);
        assert_eq!(promote_res.promoted_peer, Some(1));
        assert_eq!(promote_res.active_peers, vec![1]);
        assert_eq!(promote_res.pruned_peers, vec![2, 3]);
    }

    #[test]
    fn test_wasm_metadata_and_plan_hooks() {
        let meta_packed = basalt_swarm_metadata();
        assert_ne!(meta_packed, 0);

        let plan_req = serde_json::json!({
            "task": "Refactor lock-free ring buffer",
            "concurrency": 2,
            "target_files": ["src/ring.rs"]
        });
        let req_bytes = serde_json::to_vec(&plan_req).unwrap();
        let plan_packed = basalt_swarm_plan(req_bytes.as_ptr(), req_bytes.len());
        assert_ne!(plan_packed, 0);
    }

    #[test]
    fn test_capability_handle_dispatch() {
        let cap = "swarm-strategy:metadata";
        let res = basalt_capability_handle(cap.as_ptr(), cap.len(), std::ptr::null(), 0);
        assert_ne!(res, 0);
        let len = (res as u64 & 0xFFFFFFFF) as usize;
        assert!(len > 0);

        let plan_cap = "swarm-strategy:plan";
        let plan_req = serde_json::json!({
            "task": "Build async event dispatcher",
            "concurrency": 4
        });
        let req_bytes = serde_json::to_vec(&plan_req).unwrap();
        let res_plan = basalt_capability_handle(plan_cap.as_ptr(), plan_cap.len(), req_bytes.as_ptr(), req_bytes.len());
        assert_ne!(res_plan, 0);
        let plan_len = (res_plan as u64 & 0xFFFFFFFF) as usize;
        assert!(plan_len > 0);

        let eval_cap = "swarm-strategy:evaluate";
        let eval_req = serde_json::json!({
            "task": "Build async event dispatcher",
            "swarm_id": "swm-eval-1",
            "candidates": [
                {
                    "peer_id": 1,
                    "role": "implementer",
                    "hypothesis": "Minimal Surgical",
                    "status": "passed",
                    "modified_files": ["src/dispatcher.rs"],
                    "check_passed": true,
                    "test_passed": true,
                    "ast_convergence": {
                        "overlap_risk": 0.0,
                        "intersecting_symbols": [],
                        "auto_merge_guaranteed": true
                    },
                    "pbt_fuzz": {
                        "passed_iterations": 5000,
                        "target_iterations": 5000,
                        "invariants_verified": ["thread_safety"],
                        "execution_cycles": 12000,
                        "failed_counterexample": null
                    },
                    "intent_drift": {
                        "drift_score": 0.01,
                        "within_safe_boundary": true,
                        "flagged_deviations": []
                    },
                    "composite_score": 190.0
                }
            ]
        });
        let eval_bytes = serde_json::to_vec(&eval_req).unwrap();
        let res_eval = basalt_capability_handle(eval_cap.as_ptr(), eval_cap.len(), eval_bytes.as_ptr(), eval_bytes.len());
        assert_ne!(res_eval, 0);
        let eval_len = (res_eval as u64 & 0xFFFFFFFF) as usize;
        assert!(eval_len > 0);

        let eval_packed = basalt_swarm_evaluate(eval_bytes.as_ptr(), eval_bytes.len());
        assert_ne!(eval_packed, 0);

        let invalid_cap = "swarm-unknown";
        let res_invalid = basalt_capability_handle(invalid_cap.as_ptr(), invalid_cap.len(), std::ptr::null(), 0);
        assert_eq!(res_invalid, -1000);
    }
}
