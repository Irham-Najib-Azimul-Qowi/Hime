use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnProfile {
    pub name: String,
    pub inferred_type: String, // "numeric" | "text" | "date" | "boolean"
    pub missing_count: usize,
    pub missing_percentage: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatasetProfile {
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<ColumnProfile>,
    pub summary_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentDesign {
    pub target_variable: String,
    pub problem_type: String, // "classification" | "regression" | "clustering"
    pub baseline_models: Vec<String>,
    pub recommended_metrics: Vec<String>,
    pub validation_strategy: String,
    pub uncertainty_and_limitations: String,
    pub execution_constraints: String,
    pub model_card_markdown: String,
}

pub struct DataAiEngine;

impl DataAiEngine {
    pub fn new() -> Self {
        Self
    }

    /// Generates a structured profile from CSV raw string.
    pub fn profile_csv(&self, csv_data: &str) -> Result<DatasetProfile, String> {
        let lines: Vec<&str> = csv_data.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            return Err("Data CSV kosong atau tidak valid".to_string());
        }

        let headers: Vec<&str> = lines[0].split(',').map(|h| h.trim()).collect();
        let column_count = headers.len();
        let row_count = lines.len() - 1;

        let mut col_data: Vec<Vec<&str>> = vec![Vec::new(); column_count];
        for line in &lines[1..] {
            let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
            for (idx, field) in fields.iter().enumerate() {
                if idx < column_count {
                    col_data[idx].push(field);
                }
            }
        }

        let mut columns = Vec::new();
        for (i, &name) in headers.iter().enumerate() {
            let values = &col_data[i];
            let mut missing_count = 0;
            let mut numeric_vals = Vec::new();

            for val in values {
                if val.is_empty() || *val == "NA" || *val == "null" || *val == "NaN" {
                    missing_count += 1;
                } else if let Ok(num) = val.parse::<f64>() {
                    numeric_vals.push(num);
                }
            }

            let inferred_type = if numeric_vals.len() > values.len() / 2 {
                "numeric"
            } else if values.iter().any(|v| v.contains('-') && v.len() == 10) {
                "date"
            } else if values.iter().all(|v| *v == "true" || *v == "false" || *v == "0" || *v == "1") {
                "boolean"
            } else {
                "text"
            };

            let missing_percentage = if row_count > 0 {
                (missing_count as f64 / row_count as f64) * 100.0
            } else {
                0.0
            };

            let (min, max, mean) = if !numeric_vals.is_empty() {
                let min_val = numeric_vals.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_val = numeric_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = numeric_vals.iter().sum();
                let mean_val = sum / numeric_vals.len() as f64;
                (Some(min_val), Some(max_val), Some(mean_val))
            } else {
                (None, None, None)
            };

            columns.push(ColumnProfile {
                name: name.to_string(),
                inferred_type: inferred_type.to_string(),
                missing_count,
                missing_percentage,
                min,
                max,
                mean,
            });
        }

        // Summary markdown
        let mut md = format!(
            "### Ringkasan Profil Dataset\n- **Jumlah Baris:** {}\n- **Jumlah Kolom:** {}\n\n| Kolom | Tipe Terdeteksi | Nilai Hilang | Min | Max | Rata-rata |\n|:---|:---:|:---:|:---:|:---:|:---:|\n",
            row_count, column_count
        );

        for col in &columns {
            let min_str = col.min.map(|m| format!("{:.2}", m)).unwrap_or_else(|| "-".to_string());
            let max_str = col.max.map(|m| format!("{:.2}", m)).unwrap_or_else(|| "-".to_string());
            let mean_str = col.mean.map(|m| format!("{:.2}", m)).unwrap_or_else(|| "-".to_string());
            md.push_str(&format!(
                "| **{}** | `{}` | {} ({:.1}%) | {} | {} | {} |\n",
                col.name, col.inferred_type, col.missing_count, col.missing_percentage, min_str, max_str, mean_str
            ));
        }

        Ok(DatasetProfile {
            row_count,
            column_count,
            columns,
            summary_markdown: md,
        })
    }

    /// Designs a reproducible, anti-data-leakage machine learning experiment.
    pub fn design_experiment(
        &self,
        target_variable: &str,
        problem_type: &str,
        num_features: usize,
    ) -> ExperimentDesign {
        let (baseline_models, recommended_metrics, validation_strategy) = match problem_type.to_lowercase().as_str() {
            "regression" => (
                vec!["LinearRegression".to_string(), "RandomForestRegressor(n_estimators=100)".to_string()],
                vec!["RMSE (Root Mean Squared Error)".to_string(), "MAE (Mean Absolute Error)".to_string(), "R² Score".to_string()],
                "5-Fold Cross Validation with StandardScaling fit strictly on train split (no test leakage)".to_string(),
            ),
            "clustering" => (
                vec!["KMeans(k=3..8)".to_string(), "DBSCAN(eps=0.5)".to_string()],
                vec!["Silhouette Score".to_string(), "Davies-Bouldin Index".to_string()],
                "Elbow Method & Silhouette Analysis on normalized feature vectors".to_string(),
            ),
            _ => (
                vec!["LogisticRegression(max_iter=1000)".to_string(), "RandomForestClassifier(max_depth=8)".to_string()],
                vec!["F1-Macro".to_string(), "ROC-AUC".to_string(), "Balanced Accuracy".to_string()],
                "Stratified 5-Fold Cross Validation preserving class proportions, scaler fit per fold".to_string(),
            ),
        };

        let uncertainty = "Model predictions must include confidence intervals or prediction probabilities. Report out-of-distribution risks for data points outside feature bounds.".to_string();
        let execution_constraints = "Optimized for local CPU execution (Intel i5 without dedicated GPU). Max RAM budget 1GB; training duration capped at 120s.".to_string();

        let model_card_md = format!(
            "# MODEL CARD & EXPERIMENT SPECIFICATION\n\n## 1. Target & Problem Type\n- **Target Variable:** `{}`\n- **Problem Type:** `{}`\n- **Feature Space:** {} columns\n\n## 2. Baselines & Evaluation Metrics\n- **Baselines:** {}\n- **Primary Metrics:** {}\n\n## 3. Data Leakage Prevention\n- **Strategy:** {}\n- *Strict Rule: Fit all scalers, encoders, and imputers ONLY on training folds.*\n\n## 4. Hardware Constraints & Reporting\n- **Resource Budget:** {}\n- **Uncertainty Reporting:** {}\n",
            target_variable,
            problem_type,
            num_features,
            baseline_models.join(", "),
            recommended_metrics.join(", "),
            validation_strategy,
            execution_constraints,
            uncertainty
        );

        ExperimentDesign {
            target_variable: target_variable.to_string(),
            problem_type: problem_type.to_string(),
            baseline_models,
            recommended_metrics,
            validation_strategy,
            uncertainty_and_limitations: uncertainty,
            execution_constraints,
            model_card_markdown: model_card_md,
        }
    }
}

impl Default for DataAiEngine {
    fn default() -> Self {
        Self::new()
    }
}

