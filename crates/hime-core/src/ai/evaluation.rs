use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::specialists::{DelegationEngine, SpecialistRegistry};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationScenario {
    pub id: String,
    pub specialist_id: String,
    pub task_description: String,
    pub required_evidence_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationReport {
    pub specialist_id: String,
    pub scenarios_evaluated: usize,
    pub scenarios_passed: usize,
    pub pass_rate: f64,
    pub average_tokens: usize,
    pub average_duration_ms: usize,
    pub overall_score: f64,
    pub evaluation_summary: String,
}

pub struct EvaluationHarness {
    registry: Arc<SpecialistRegistry>,
    delegation: Arc<DelegationEngine>,
}

impl EvaluationHarness {
    pub fn new(registry: Arc<SpecialistRegistry>, delegation: Arc<DelegationEngine>) -> Self {
        Self {
            registry,
            delegation,
        }
    }

    /// Runs automated regression and capability evaluation on a specialist.
    pub fn evaluate_specialist(&self, specialist_id: &str) -> Result<EvaluationReport, String> {
        let spec = self
            .registry
            .get_specialist(specialist_id)
            .ok_or_else(|| format!("Specialist '{}' tidak ditemukan.", specialist_id))?;

        let scenarios = self.get_scenarios_for_specialist(specialist_id);
        if scenarios.is_empty() {
            return Err(format!("Tidak ada skenario evaluasi untuk '{}'.", specialist_id));
        }

        let mut passed = 0;
        let mut total_tokens = 0;
        let mut total_duration = 0;

        for scenario in &scenarios {
            let res = self.delegation.delegate_task(
                specialist_id,
                &format!("eval-{}", scenario.id),
                &scenario.task_description,
            )?;

            total_tokens += res.tokens_used as usize;
            total_duration += res.duration_ms as usize;

            let has_required_evidence = scenario.required_evidence_keywords.iter().all(|kw| {
                res.evidence.iter().any(|ev| ev.to_lowercase().contains(&kw.to_lowercase()))
            });

            if res.status == "completed" && has_required_evidence {
                passed += 1;
            }
        }

        let count = scenarios.len();
        let pass_rate = (passed as f64 / count as f64) * 100.0;
        let avg_tokens = total_tokens / count;
        let avg_duration = total_duration / count;

        // Overall score factoring pass rate, evidence reliability, and token efficiency
        let efficiency_bonus = if avg_tokens < spec.resource_budget.max_tokens / 2 { 5.0 } else { 0.0 };
        let overall_score = (pass_rate * 0.95 + efficiency_bonus).min(100.0);

        // Update score in registry
        self.registry.update_evaluation_score(specialist_id, overall_score);

        info!(
            specialist = %specialist_id,
            score = %overall_score,
            pass_rate = %pass_rate,
            "Specialist capability evaluation completed"
        );

        let summary = format!(
            "Evaluasi Specialist '{}' ({}) selesai dengan tingkat kelulusan {:.1}% ({} dari {} skenario) dan skor keseluruhan {:.1}/100.",
            spec.name, spec.role_title, pass_rate, passed, count, overall_score
        );

        Ok(EvaluationReport {
            specialist_id: specialist_id.to_string(),
            scenarios_evaluated: count,
            scenarios_passed: passed,
            pass_rate,
            average_tokens: avg_tokens,
            average_duration_ms: avg_duration,
            overall_score,
            evaluation_summary: summary,
        })
    }

    fn get_scenarios_for_specialist(&self, specialist_id: &str) -> Vec<EvaluationScenario> {
        match specialist_id {
            "specialist.software_engineer" => vec![
                EvaluationScenario {
                    id: "se-1".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Audit working tree dan validasi path boundary".to_string(),
                    required_evidence_keywords: vec!["Git".to_string(), "Allowlist".to_string()],
                },
                EvaluationScenario {
                    id: "se-2".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Jalankan skrip pengujian sandboxed tanpa injeksi".to_string(),
                    required_evidence_keywords: vec!["Allowlist".to_string()],
                },
            ],
            "specialist.data_ai" => vec![
                EvaluationScenario {
                    id: "data-1".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Rancang baseline klasifikasi anti-data-leakage".to_string(),
                    required_evidence_keywords: vec!["kebocoran data".to_string(), "RAM".to_string()],
                },
                EvaluationScenario {
                    id: "data-2".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Evaluasi dataset tabular dan batasan CPU".to_string(),
                    required_evidence_keywords: vec!["CPU".to_string()],
                },
            ],
            "specialist.research" => vec![
                EvaluationScenario {
                    id: "res-1".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Lakukan pencarian web publik dan validasi SSRF".to_string(),
                    required_evidence_keywords: vec!["SSRF".to_string(), "untrusted_web_content".to_string()],
                },
            ],
            "specialist.document" => vec![
                EvaluationScenario {
                    id: "doc-1".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Format proposal skripsi standar academic-id".to_string(),
                    required_evidence_keywords: vec!["academic-id".to_string(), "visual".to_string()],
                },
            ],
            "specialist.productivity" => vec![
                EvaluationScenario {
                    id: "prod-1".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Rancang jadwal belajar 45 menit dengan jeda 10 menit".to_string(),
                    required_evidence_keywords: vec!["Collision".to_string(), "Non-punitive".to_string()],
                },
            ],
            _ => vec![
                EvaluationScenario {
                    id: "stor-1".to_string(),
                    specialist_id: specialist_id.to_string(),
                    task_description: "Pindai duplikat file dan siapkan pemulihan reversibel".to_string(),
                    required_evidence_keywords: vec!["Staging".to_string(), "Audit".to_string()],
                },
            ],
        }
    }
}

