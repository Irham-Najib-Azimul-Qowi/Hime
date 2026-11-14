import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  listSpecialists,
  delegateToSpecialist,
  listSpecialistDelegations,
  evaluateSpecialist,
  listProactiveSuggestions,
  evaluateProactiveSuggestions,
  respondProactiveSuggestion,
  getProactiveSettings,
  setProactiveSettings,
  profileDataset,
  designExperiment,
} from '../../services/ipcService';
import {
  SpecialistDefinition,
  DelegationResult,
  DatasetProfile,
  ExperimentDesign,
  ProactiveSuggestionItem,
  ProactiveSettings,
  EvaluationReport,
} from '../../types/specialists';

export const SpecialistsView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'registry' | 'delegation' | 'data_ai' | 'proactive'>('registry');
  const [specialists, setSpecialists] = useState<SpecialistDefinition[]>([]);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // Delegation state
  const [selectedSpecialistId, setSelectedSpecialistId] = useState<string>('specialist.software_engineer');
  const [delegationPrompt, setDelegationPrompt] = useState('');
  const [delegations, setDelegations] = useState<DelegationResult[]>([]);
  const [latestDelegation, setLatestDelegation] = useState<DelegationResult | null>(null);
  const [delegating, setDelegating] = useState(false);

  // Evaluation state
  const [evalReport, setEvalReport] = useState<EvaluationReport | null>(null);
  const [evaluating, setEvaluating] = useState(false);

  // Data & AI state
  const [csvInput, setCsvInput] = useState(
    'student_id,study_hours,previous_score,passed\n101,4.5,78,1\n102,1.2,52,0\n103,5.8,,1\n104,3.0,65,1\n105,,48,0'
  );
  const [datasetProfile, setDatasetProfile] = useState<DatasetProfile | null>(null);
  const [targetVar, setTargetVar] = useState('passed');
  const [problemType, setProblemType] = useState('classification');
  const [experimentDesign, setExperimentDesign] = useState<ExperimentDesign | null>(null);
  const [profiling, setProfiling] = useState(false);

  // Proactive state
  const [proactiveSettings, setProactiveSettingsState] = useState<ProactiveSettings | null>(null);
  const [suggestions, setSuggestions] = useState<ProactiveSuggestionItem[]>([]);

  const loadInitialData = useCallback(async () => {
    try {
      const [specs, dels, settings, sugs] = await Promise.all([
        listSpecialists(false),
        listSpecialistDelegations(undefined, 10),
        getProactiveSettings(),
        listProactiveSuggestions('pending', 10),
      ]);
      setSpecialists(specs);
      setDelegations(dels);
      setProactiveSettingsState(settings);
      setSuggestions(sugs);
    } catch (err: any) {
      console.warn('Initial load error:', err);
    }
  }, []);

  useEffect(() => {
    loadInitialData();
  }, [loadInitialData]);

  // Delegation action
  const handleDelegate = async () => {
    if (!delegationPrompt.trim() || !selectedSpecialistId) return;
    setDelegating(true);
    setStatusMessage(null);
    try {
      const taskId = `task-${Date.now()}`;
      const res = await delegateToSpecialist(selectedSpecialistId, taskId, delegationPrompt.trim());
      setLatestDelegation(res);
      setDelegations((prev) => [res, ...prev]);
      setStatusMessage(`Delegasi selesai: ${res.output_summary}`);
      setDelegationPrompt('');
    } catch (err: any) {
      setStatusMessage(`Delegasi gagal / ditolak: ${err.message || String(err)}`);
    } finally {
      setDelegating(false);
    }
  };

  // Evaluation action
  const handleEvaluate = async (specId: string) => {
    setEvaluating(true);
    setStatusMessage(null);
    try {
      const report = await evaluateSpecialist(specId);
      setEvalReport(report);
      // Refresh specialists to reflect new score
      const updatedSpecs = await listSpecialists(false);
      setSpecialists(updatedSpecs);
      setStatusMessage(report.evaluation_summary);
    } catch (err: any) {
      setStatusMessage(`Evaluasi gagal: ${err.message || String(err)}`);
    } finally {
      setEvaluating(false);
    }
  };

  // Data & AI actions
  const handleProfileDataset = async () => {
    if (!csvInput.trim()) return;
    setProfiling(true);
    setStatusMessage(null);
    try {
      const profile = await profileDataset(csvInput.trim());
      setDatasetProfile(profile);
      setStatusMessage(`Dataset diprofilkan: ${profile.row_count} baris, ${profile.column_count} kolom.`);
    } catch (err: any) {
      setStatusMessage(`Gagal profil dataset: ${err.message || String(err)}`);
    } finally {
      setProfiling(false);
    }
  };

  const handleDesignExperiment = async () => {
    try {
      const numCols = datasetProfile ? datasetProfile.column_count : 4;
      const design = await designExperiment(targetVar.trim(), problemType, numCols);
      setExperimentDesign(design);
      setStatusMessage(`Rancangan eksperimen ML '${targetVar}' berhasil dibuat.`);
    } catch (err: any) {
      setStatusMessage(`Gagal membuat eksperimen: ${err.message || String(err)}`);
    }
  };

  // Proactive actions
  const handleToggleProactive = async () => {
    if (!proactiveSettings) return;
    const newEnabled = !proactiveSettings.is_enabled;
    try {
      await setProactiveSettings(newEnabled, proactiveSettings.cooldown_minutes);
      setProactiveSettingsState((prev) => (prev ? { ...prev, is_enabled: newEnabled } : null));
      setStatusMessage(`Bantuan proaktif ${newEnabled ? 'diaktifkan' : 'dinonaktifkan'}.`);
    } catch (err: any) {
      setStatusMessage(`Gagal mengubah pengaturan: ${err.message || String(err)}`);
    }
  };

  const handleMuteProactive = async (hours: number) => {
    if (!proactiveSettings) return;
    try {
      await setProactiveSettings(proactiveSettings.is_enabled, proactiveSettings.cooldown_minutes, hours);
      const updated = await getProactiveSettings();
      setProactiveSettingsState(updated);
      setStatusMessage(`Bantuan proaktif dibisukan selama ${hours} jam.`);
    } catch (err: any) {
      setStatusMessage(`Gagal membisukan proaktif: ${err.message || String(err)}`);
    }
  };

  const handleRespondSuggestion = async (id: string, action: string) => {
    try {
      await respondProactiveSuggestion(id, action);
      setSuggestions((prev) => prev.filter((s) => s.id !== id));
      setStatusMessage(`Saran proaktif '${action}' berhasil diproses.`);
    } catch (err: any) {
      setStatusMessage(`Gagal merespon saran: ${err.message || String(err)}`);
    }
  };

  const handleTriggerProactiveCheck = async () => {
    try {
      const fresh = await evaluateProactiveSuggestions();
      setSuggestions(fresh);
      setStatusMessage(`Evaluasi proaktif selesai: ${fresh.length} saran aktif.`);
    } catch (err: any) {
      setStatusMessage(`Evaluasi proaktif gagal: ${err.message || String(err)}`);
    }
  };

  return (
    <div className="flex flex-col h-full space-y-4 p-4 text-slate-100 overflow-y-auto">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-700/60 pb-3">
        <div>
          <h1 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <span>Specialist Agents, Skills & Proactive Assistance</span>
            <Badge variant="success" className="text-xs">
              Quality Bar Verified
            </Badge>
          </h1>
          <p className="text-xs text-slate-400">
            Registri specialist terdefinisi (Software, Data/AI, Research, Document, Productivity, Storage), delegasi terukur dengan bukti, dan bantuan proaktif opt-in.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Badge variant="violet" className="text-xs">
            {specialists.length} Specialists Terdaftar
          </Badge>
        </div>
      </div>

      {/* Status Message Alert */}
      {statusMessage && (
        <div className="text-xs p-2.5 rounded-lg bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 flex items-center justify-between">
          <span>{statusMessage}</span>
          <button onClick={() => setStatusMessage(null)} className="text-slate-400 hover:text-white ml-2">
            &times;
          </button>
        </div>
      )}

      {/* Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-700/60 pb-2">
        <button
          onClick={() => setActiveTab('registry')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'registry'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Registri Specialist ({specialists.length})
        </button>
        <button
          onClick={() => setActiveTab('delegation')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'delegation'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Delegasi & Bukti Eksekusi
        </button>
        <button
          onClick={() => setActiveTab('data_ai')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'data_ai'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Data & AI Studio (UR-13)
        </button>
        <button
          onClick={() => setActiveTab('proactive')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'proactive'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Bantuan Proaktif Aman
        </button>
      </div>

      {/* Tab 1: Specialist Registry */}
      {activeTab === 'registry' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {specialists.map((spec) => (
              <Card key={spec.id} className="p-3 bg-slate-800/60 border-slate-700 flex flex-col justify-between space-y-3">
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-[10px] font-mono text-indigo-400 uppercase tracking-wider">{spec.domain}</span>
                    <Badge variant={spec.evaluation_score >= 90 ? 'success' : 'warning'} className="text-[10px]">
                      Skor: {spec.evaluation_score.toFixed(1)}/100
                    </Badge>
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold text-white">{spec.name}</h3>
                    <div className="text-[11px] text-slate-400">{spec.role_title}</div>
                  </div>
                  <div className="space-y-1">
                    <span className="text-[10px] font-medium text-slate-400 uppercase">Tanggung Jawab:</span>
                    <ul className="text-[11px] text-slate-300 space-y-0.5 list-disc list-inside">
                      {spec.responsibilities.slice(0, 2).map((r, idx) => (
                        <li key={idx} className="truncate">{r}</li>
                      ))}
                    </ul>
                  </div>
                  <div className="text-[10px] text-slate-400">
                    <span className="font-semibold text-slate-300">Allowed Tools: </span>
                    <span className="font-mono text-indigo-300">{spec.allowed_tools.join(', ')}</span>
                  </div>
                </div>

                <div className="pt-2 border-t border-slate-700/60 flex items-center justify-between">
                  <span className="text-[10px] text-slate-400">Budget: {spec.resource_budget.max_tokens} toks</span>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => handleEvaluate(spec.id)}
                    disabled={evaluating}
                    className="text-[11px] py-1 px-2"
                  >
                    {evaluating ? 'Menguji...' : 'Uji Evaluasi'}
                  </Button>
                </div>
              </Card>
            ))}
          </div>

          {/* Evaluation Report Modal / Box */}
          {evalReport && (
            <Card className="p-4 bg-slate-900 border-slate-800 space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="text-xs font-semibold text-indigo-400">Laporan Evaluasi Specialist Otomatis</span>
                <Badge variant="success" className="text-[10px]">
                  Skor Akhir: {evalReport.overall_score.toFixed(1)} / 100
                </Badge>
              </div>
              <p className="text-xs text-slate-300 leading-relaxed">{evalReport.evaluation_summary}</p>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-2 pt-2 text-[11px]">
                <div className="bg-slate-950 p-2 rounded">
                  <span className="text-slate-500 block">Skenario Diuji:</span>
                  <span className="font-semibold text-white">{evalReport.scenarios_evaluated}</span>
                </div>
                <div className="bg-slate-950 p-2 rounded">
                  <span className="text-slate-500 block">Kelulusan:</span>
                  <span className="font-semibold text-emerald-400">{evalReport.pass_rate.toFixed(0)}%</span>
                </div>
                <div className="bg-slate-950 p-2 rounded">
                  <span className="text-slate-500 block">Rata-rata Token:</span>
                  <span className="font-semibold text-white">{evalReport.average_tokens}</span>
                </div>
                <div className="bg-slate-950 p-2 rounded">
                  <span className="text-slate-500 block">Durasi:</span>
                  <span className="font-semibold text-white">{evalReport.average_duration_ms} ms</span>
                </div>
              </div>
            </Card>
          )}
        </div>
      )}

      {/* Tab 2: Delegation & Task Runner */}
      {activeTab === 'delegation' && (
        <div className="space-y-4">
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-3">
            <h3 className="text-xs font-semibold text-slate-200">Kirim Tugas ke Domain Specialist</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Pilih Specialist</label>
                <select
                  value={selectedSpecialistId}
                  onChange={(e) => setSelectedSpecialistId(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
                >
                  {specialists.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name} ({s.domain})
                    </option>
                  ))}
                </select>
              </div>
              <div className="md:col-span-2">
                <label className="text-[11px] text-slate-400 block mb-1">Instruksi / Prompt Spesialis</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    placeholder="Contoh: Audit sanitasi input dan boundary direktori"
                    value={delegationPrompt}
                    onChange={(e) => setDelegationPrompt(e.target.value)}
                    className="flex-1 bg-slate-900 border border-slate-700 rounded px-3 py-1.5 text-xs text-white"
                  />
                  <Button size="sm" onClick={handleDelegate} disabled={delegating || !delegationPrompt.trim()} className="text-xs">
                    {delegating ? 'Memproses...' : 'Delegasikan'}
                  </Button>
                </div>
              </div>
            </div>
          </Card>

          {/* Latest Execution Card */}
          {latestDelegation && (
            <Card className="p-3 bg-slate-900 border-slate-800 space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="text-xs font-semibold text-emerald-400">Hasil Delegasi Terakhir (Verified Evidence)</span>
                <Badge variant="success" className="text-[10px]">
                  {latestDelegation.status.toUpperCase()}
                </Badge>
              </div>
              <p className="text-xs text-slate-200">{latestDelegation.output_summary}</p>
              <div className="space-y-1 pt-1">
                <span className="text-[11px] font-semibold text-slate-400">Bukti Nyata (Checklist):</span>
                <div className="space-y-1">
                  {latestDelegation.evidence.map((ev, i) => (
                    <div key={i} className="flex items-center gap-1.5 text-xs text-slate-300 bg-slate-950 px-2 py-1 rounded">
                      <span className="text-emerald-400 font-bold">✓</span>
                      <span>{ev}</span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="flex items-center justify-between text-[10px] text-slate-500 pt-1">
                <span>Tokens: {latestDelegation.tokens_used}</span>
                <span>Durasi: {latestDelegation.duration_ms} ms</span>
              </div>
            </Card>
          )}

          {/* Delegation History Table */}
          <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2">
            <h3 className="text-xs font-semibold text-slate-200">Riwayat Delegasi</h3>
            <div className="space-y-1.5 max-h-56 overflow-y-auto">
              {delegations.length === 0 ? (
                <p className="text-slate-500 text-xs italic">Belum ada delegasi yang tercatat.</p>
              ) : (
                delegations.map((d) => (
                  <div key={d.delegation_id} className="p-2 bg-slate-900/60 rounded border border-slate-800 flex items-center justify-between text-xs">
                    <div>
                      <div className="font-semibold text-slate-200">{d.output_summary}</div>
                      <div className="text-[10px] text-slate-400">Specialist: {d.specialist_id}</div>
                    </div>
                    <div className="text-right">
                      <Badge variant="cyan" className="text-[10px]">{d.status}</Badge>
                      <div className="text-[10px] text-slate-500">{d.tokens_used} toks • {d.duration_ms} ms</div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </Card>
        </div>
      )}

      {/* Tab 3: Data & AI Studio (UR-13) */}
      {activeTab === 'data_ai' && (
        <div className="space-y-4">
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-slate-200">Profil & Integritas Dataset (CSV Input)</span>
              <span className="text-[11px] text-slate-400">Pemeriksaan missing values, tipe data, dan statistik deskriptif</span>
            </div>
            <textarea
              value={csvInput}
              onChange={(e) => setCsvInput(e.target.value)}
              rows={4}
              className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-xs font-mono text-white"
            />
            <div className="flex justify-end gap-2">
              <Button size="sm" onClick={handleProfileDataset} disabled={profiling} className="text-xs">
                {profiling ? 'Memproses...' : 'Profilkan Dataset'}
              </Button>
            </div>
          </Card>

          {/* Dataset Profile Output */}
          {datasetProfile && (
            <Card className="p-3 bg-slate-900 border-slate-800 space-y-3">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="text-xs font-semibold text-emerald-400">
                  Ringkasan Kolom & Statistik ({datasetProfile.row_count} baris, {datasetProfile.column_count} kolom)
                </span>
                <Badge variant="success" className="text-[10px]">Valid Profiling</Badge>
              </div>

              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs font-mono">
                  <thead>
                    <tr className="border-b border-slate-800 text-slate-400">
                      <th className="p-1.5">Nama Kolom</th>
                      <th className="p-1.5">Tipe</th>
                      <th className="p-1.5">Missing</th>
                      <th className="p-1.5">Min</th>
                      <th className="p-1.5">Max</th>
                      <th className="p-1.5">Rata-rata</th>
                    </tr>
                  </thead>
                  <tbody>
                    {datasetProfile.columns.map((col, idx) => (
                      <tr key={idx} className="border-b border-slate-800/40 hover:bg-slate-800/20">
                        <td className="p-1.5 font-semibold text-white">{col.name}</td>
                        <td className="p-1.5 text-indigo-400">{col.inferred_type}</td>
                        <td className="p-1.5 text-amber-400">{col.missing_count} ({col.missing_percentage.toFixed(1)}%)</td>
                        <td className="p-1.5 text-slate-300">{col.min !== null ? col.min.toFixed(2) : '-'}</td>
                        <td className="p-1.5 text-slate-300">{col.max !== null ? col.max.toFixed(2) : '-'}</td>
                        <td className="p-1.5 text-slate-300">{col.mean !== null ? col.mean.toFixed(2) : '-'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Experiment Designer Controls */}
              <div className="pt-3 border-t border-slate-800 space-y-2">
                <h4 className="text-xs font-semibold text-slate-200">Perancangan Eksperimen Machine Learning</h4>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
                  <div>
                    <label className="text-[11px] text-slate-400 block mb-1">Target Variable</label>
                    <input
                      type="text"
                      value={targetVar}
                      onChange={(e) => setTargetVar(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs text-white"
                    />
                  </div>
                  <div>
                    <label className="text-[11px] text-slate-400 block mb-1">Tipe Masalah</label>
                    <select
                      value={problemType}
                      onChange={(e) => setProblemType(e.target.value)}
                      className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs text-white"
                    >
                      <option value="classification">Classification (F1-score, ROC-AUC)</option>
                      <option value="regression">Regression (RMSE, MAE, R²)</option>
                      <option value="clustering">Clustering (Silhouette Score)</option>
                    </select>
                  </div>
                  <div className="flex items-end">
                    <Button size="sm" onClick={handleDesignExperiment} className="w-full text-xs">
                      Rancang Model Card
                    </Button>
                  </div>
                </div>
              </div>
            </Card>
          )}

          {/* Model Card Output */}
          {experimentDesign && (
            <Card className="p-3 bg-slate-950 border-slate-800 space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="text-xs font-semibold text-indigo-400">Model Card & Rencana Eksperimen Anti-Leakage</span>
                <Badge variant="violet" className="text-[10px]">CPU Friendly (i5)</Badge>
              </div>
              <pre className="text-xs font-mono text-slate-300 bg-black/60 p-3 rounded overflow-x-auto max-h-72 whitespace-pre-wrap leading-relaxed border border-slate-900">
                {experimentDesign.model_card_markdown}
              </pre>
            </Card>
          )}
        </div>
      )}

      {/* Tab 4: Safe Proactive Assistance */}
      {activeTab === 'proactive' && (
        <div className="space-y-4">
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-3">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-xs font-semibold text-slate-200">Kontrol Bantuan Proaktif (Opt-In & Rate-Limited)</h3>
                <p className="text-[11px] text-slate-400">
                  Hime hanya memberikan saran non-intrusif dan tidak pernah menjalankan tindakan berisiko tanpa izin.
                </p>
              </div>
              <Button
                size="sm"
                variant={proactiveSettings?.is_enabled ? 'danger' : 'primary'}
                onClick={handleToggleProactive}
                className="text-xs"
              >
                {proactiveSettings?.is_enabled ? 'Nonaktifkan' : 'Aktifkan'}
              </Button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-3 pt-2 border-t border-slate-700">
              <div className="bg-slate-900/60 p-2.5 rounded border border-slate-800">
                <span className="text-[11px] text-slate-400 block mb-1">Status Proaktif:</span>
                <Badge variant={proactiveSettings?.is_enabled ? 'success' : 'default'} className="text-xs">
                  {proactiveSettings?.is_enabled ? 'Aktif (Opt-in)' : 'Nonaktif'}
                </Badge>
              </div>
              <div className="bg-slate-900/60 p-2.5 rounded border border-slate-800">
                <span className="text-[11px] text-slate-400 block mb-1">Cooldown Antar-Saran:</span>
                <span className="text-xs font-semibold text-slate-200">
                  {proactiveSettings?.cooldown_minutes} Menit
                </span>
              </div>
              <div className="bg-slate-900/60 p-2.5 rounded border border-slate-800">
                <span className="text-[11px] text-slate-400 block mb-1">Mode Mute Cepat:</span>
                <div className="flex gap-1.5">
                  <Button size="sm" variant="secondary" onClick={() => handleMuteProactive(1)} className="text-[10px] py-1 px-1.5">
                    1 Jam
                  </Button>
                  <Button size="sm" variant="secondary" onClick={() => handleMuteProactive(4)} className="text-[10px] py-1 px-1.5">
                    4 Jam
                  </Button>
                  <Button size="sm" variant="secondary" onClick={() => handleMuteProactive(24)} className="text-[10px] py-1 px-1.5">
                    24 Jam
                  </Button>
                </div>
              </div>
            </div>
          </Card>

          {/* Pending Suggestions List */}
          <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-semibold text-slate-200">Saran & Rekomendasi Terkini</h3>
              <Button size="sm" variant="ghost" onClick={handleTriggerProactiveCheck} className="text-xs">
                Periksa Saran Baru
              </Button>
            </div>

            <div className="space-y-2 max-h-72 overflow-y-auto">
              {suggestions.length === 0 ? (
                <p className="text-slate-500 text-xs italic">Tidak ada saran proaktif saat ini.</p>
              ) : (
                suggestions.map((sug) => (
                  <div key={sug.id} className="p-3 bg-slate-900 rounded-lg border border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-2">
                    <div>
                      <div className="flex items-center gap-2">
                        <Badge variant="cyan" className="text-[10px] uppercase">{sug.category}</Badge>
                        <span className="text-xs font-semibold text-white">{sug.title}</span>
                      </div>
                      <p className="text-xs text-slate-300 mt-1">{sug.content}</p>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button size="sm" onClick={() => handleRespondSuggestion(sug.id, 'accept')} className="text-xs">
                        Terima
                      </Button>
                      <Button size="sm" variant="secondary" onClick={() => handleRespondSuggestion(sug.id, 'dismiss')} className="text-xs">
                        Tolak
                      </Button>
                      <Button size="sm" variant="ghost" onClick={() => handleRespondSuggestion(sug.id, 'mute')} className="text-xs">
                        Snooze
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </Card>
        </div>
      )}
    </div>
  );
};
