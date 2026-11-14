import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  listDeveloperWorkspaces,
  registerDeveloperWorkspace,
  runDeveloperCommand,
  getGitStatus,
  getGitDiff,
  commitGitChanges,
  listSeTemplates,
  renderSeTemplate,
} from '../../services/ipcService';
import {
  DeveloperWorkspace,
  DeveloperCommandOutput,
  GitStatusResult,
  SeTemplateItem,
} from '../../types/developer';

export const DeveloperWorkspaceView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'terminal' | 'git' | 'templates'>('git');
  const [workspaces, setWorkspaces] = useState<DeveloperWorkspace[]>([]);
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // New workspace form
  const [showAddWs, setShowAddWs] = useState(false);
  const [newWsName, setNewWsName] = useState('');
  const [newWsPath, setNewWsPath] = useState('');

  // Terminal state
  const [commandStr, setCommandStr] = useState('cargo check --workspace');
  const [runningCmd, setRunningCmd] = useState(false);
  const [terminalOutput, setTerminalOutput] = useState<DeveloperCommandOutput | null>(null);

  // Git state
  const [gitStatus, setGitStatus] = useState<GitStatusResult | null>(null);
  const [gitDiff, setGitDiff] = useState<string>('');
  const [commitMsg, setCommitMsg] = useState('');
  const [committing, setCommitting] = useState(false);

  // SE Templates state
  const [templates, setTemplates] = useState<SeTemplateItem[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>('skpl-ieee830');
  const [projectName, setProjectName] = useState('Hime Personal AI');
  const [authorName, setAuthorName] = useState('Principal Engineer');
  const [renderedTemplate, setRenderedTemplate] = useState<string>('');

  const loadWorkspaces = useCallback(async () => {
    try {
      const wsList = await listDeveloperWorkspaces();
      setWorkspaces(wsList);
      if (wsList.length > 0 && !selectedWorkspaceId) {
        setSelectedWorkspaceId(wsList[0].id);
      }
    } catch (err: any) {
      setStatusMessage(`Gagal memuat workspace: ${err.message || String(err)}`);
    }
  }, [selectedWorkspaceId]);

  const loadGitInfo = useCallback(async () => {
    if (!selectedWorkspaceId) return;
    try {
      const [status, diffRes] = await Promise.all([
        getGitStatus(selectedWorkspaceId),
        getGitDiff(selectedWorkspaceId, false),
      ]);
      setGitStatus(status);
      setGitDiff(diffRes.diff);
    } catch (err: any) {
      console.warn('Git info load error:', err);
    }
  }, [selectedWorkspaceId]);

  const loadTemplates = useCallback(async () => {
    try {
      const tpls = await listSeTemplates();
      setTemplates(tpls);
    } catch (err: any) {
      console.warn('Templates load error:', err);
    }
  }, []);

  useEffect(() => {
    loadWorkspaces();
    loadTemplates();
  }, [loadWorkspaces, loadTemplates]);

  useEffect(() => {
    if (selectedWorkspaceId && activeTab === 'git') {
      loadGitInfo();
    }
  }, [selectedWorkspaceId, activeTab, loadGitInfo]);

  const handleRegisterWorkspace = async () => {
    if (!newWsName.trim() || !newWsPath.trim()) return;
    setLoading(true);
    try {
      const created = await registerDeveloperWorkspace(newWsName.trim(), newWsPath.trim());
      setWorkspaces((prev) => [...prev, created]);
      setSelectedWorkspaceId(created.id);
      setShowAddWs(false);
      setNewWsName('');
      setNewWsPath('');
      setStatusMessage(`Workspace '${created.name}' berhasil didaftarkan.`);
    } catch (err: any) {
      setStatusMessage(`Registrasi workspace gagal: ${err.message || String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleRunCommand = async () => {
    if (!commandStr.trim() || !selectedWorkspaceId) return;
    setRunningCmd(true);
    setStatusMessage(null);
    try {
      const parts = commandStr.trim().split(/\s+/);
      const binary = parts[0];
      const args = parts.slice(1);
      const res = await runDeveloperCommand(selectedWorkspaceId, binary, args);
      setTerminalOutput(res);
    } catch (err: any) {
      setStatusMessage(`Eksekusi command ditolak / gagal: ${err.message || String(err)}`);
    } finally {
      setRunningCmd(false);
    }
  };

  const handleCommit = async () => {
    if (!commitMsg.trim() || !selectedWorkspaceId) return;
    setCommitting(true);
    setStatusMessage(null);
    try {
      const res = await commitGitChanges(selectedWorkspaceId, commitMsg.trim());
      if (res.success) {
        setStatusMessage(`Commit berhasil dengan hash: ${res.commit_hash}`);
        setCommitMsg('');
        loadGitInfo();
      }
    } catch (err: any) {
      setStatusMessage(`Commit gagal (pemeriksaan secret atau boundary error): ${err.message || String(err)}`);
    } finally {
      setCommitting(false);
    }
  };

  const handleRenderTemplate = async () => {
    if (!selectedTemplate) return;
    setLoading(true);
    try {
      const res = await renderSeTemplate(selectedTemplate, projectName, authorName);
      setRenderedTemplate(res.rendered_markdown);
    } catch (err: any) {
      setStatusMessage(`Gagal render template: ${err.message || String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const activeWs = workspaces.find((w) => w.id === selectedWorkspaceId);

  return (
    <div className="flex flex-col h-full space-y-4 p-4 text-slate-100 overflow-y-auto">
      {/* Header & Workspace Switcher */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-700/60 pb-3">
        <div>
          <h1 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <span>Developer Workspace & Engineering Controls</span>
            <Badge variant="success" className="text-xs">
              Sandboxed PEP
            </Badge>
          </h1>
          <p className="text-xs text-slate-400">
            Perintah sandboxed terverifikasi, inspeksi Git aman dengan pre-commit secret scanner, dan standar rekayasa IEEE/ISO.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <select
            value={selectedWorkspaceId}
            onChange={(e) => setSelectedWorkspaceId(e.target.value)}
            className="bg-slate-800 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          >
            {workspaces.map((ws) => (
              <option key={ws.id} value={ws.id}>
                {ws.name} ({ws.root_path})
              </option>
            ))}
          </select>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setShowAddWs(!showAddWs)}
            className="text-xs"
          >
            {showAddWs ? 'Batal' : '+ Tambah Workspace'}
          </Button>
        </div>
      </div>

      {/* Add Workspace Modal / Expansion */}
      {showAddWs && (
        <Card className="p-3 bg-slate-800/80 border-slate-700 space-y-2">
          <h3 className="text-xs font-semibold text-slate-200">Daftarkan Workspace Developer Baru</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            <input
              type="text"
              placeholder="Nama Workspace (misal: Hime Core)"
              value={newWsName}
              onChange={(e) => setNewWsName(e.target.value)}
              className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
            />
            <input
              type="text"
              placeholder="Path Direktori Absolut (misal: X:\folder_ai\Hime)"
              value={newWsPath}
              onChange={(e) => setNewWsPath(e.target.value)}
              className="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
            />
          </div>
          <div className="flex justify-end gap-2 pt-1">
            <Button size="sm" onClick={handleRegisterWorkspace} disabled={loading} className="text-xs">
              Simpan Workspace
            </Button>
          </div>
        </Card>
      )}

      {/* Active Workspace Info Pill */}
      {activeWs && (
        <div className="flex items-center justify-between text-xs bg-slate-800/50 border border-slate-700/50 rounded-lg px-3 py-2">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-400"></span>
            <span className="font-medium text-slate-300">Root Terikat:</span>
            <span className="font-mono text-slate-400">{activeWs.root_path}</span>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant="default" className="text-[10px]">
              Anti-Traversal Active (No ADS, No UNC Bypass)
            </Badge>
          </div>
        </div>
      )}

      {/* Status Message Alert */}
      {statusMessage && (
        <div className="text-xs p-2.5 rounded-lg bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 flex items-center justify-between">
          <span>{statusMessage}</span>
          <button onClick={() => setStatusMessage(null)} className="text-slate-400 hover:text-white ml-2">
            &times;
          </button>
        </div>
      )}

      {/* Navigation Sub-Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-700/60 pb-2">
        <button
          onClick={() => setActiveTab('git')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'git'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Git & Secret Security
        </button>
        <button
          onClick={() => setActiveTab('terminal')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'terminal'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Sandboxed Command Terminal
        </button>
        <button
          onClick={() => setActiveTab('templates')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'templates'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Standar Rekayasa (SKPL / DPPL)
        </button>
      </div>

      {/* Tab 1: Git & Secret Security */}
      {activeTab === 'git' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
            <Card className="p-3 bg-slate-800/60 border-slate-700">
              <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Branch Aktif</span>
              <span className="text-sm font-mono font-semibold text-indigo-400">
                {gitStatus?.branch || 'Memeriksa...'}
              </span>
            </Card>
            <Card className="p-3 bg-slate-800/60 border-slate-700">
              <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Staged Changes</span>
              <span className="text-sm font-semibold text-emerald-400">
                {gitStatus?.staged.length || 0} file
              </span>
            </Card>
            <Card className="p-3 bg-slate-800/60 border-slate-700">
              <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Modified Unstaged</span>
              <span className="text-sm font-semibold text-amber-400">
                {gitStatus?.modified.length || 0} file
              </span>
            </Card>
            <Card className="p-3 bg-slate-800/60 border-slate-700">
              <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Untracked Files</span>
              <span className="text-sm font-semibold text-slate-300">
                {gitStatus?.untracked.length || 0} file
              </span>
            </Card>
          </div>

          {/* Modified & Untracked Lists */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2">
              <h3 className="text-xs font-semibold text-amber-400">Modified Files</h3>
              <div className="max-h-36 overflow-y-auto space-y-1 font-mono text-[11px]">
                {gitStatus?.modified.length === 0 ? (
                  <p className="text-slate-500 italic">Tidak ada modifikasi file.</p>
                ) : (
                  gitStatus?.modified.map((file, i) => (
                    <div key={i} className="text-slate-300 bg-slate-900/60 px-2 py-0.5 rounded truncate">
                      M {file}
                    </div>
                  ))
                )}
              </div>
            </Card>
            <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2">
              <h3 className="text-xs font-semibold text-slate-300">Untracked Files</h3>
              <div className="max-h-36 overflow-y-auto space-y-1 font-mono text-[11px]">
                {gitStatus?.untracked.length === 0 ? (
                  <p className="text-slate-500 italic">Tidak ada untracked file.</p>
                ) : (
                  gitStatus?.untracked.map((file, i) => (
                    <div key={i} className="text-slate-400 bg-slate-900/60 px-2 py-0.5 rounded truncate">
                      ? {file}
                    </div>
                  ))
                )}
              </div>
            </Card>
          </div>

          {/* Unified Diff View */}
          <Card className="p-3 bg-slate-900 border-slate-800 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-slate-300">Working Tree Unified Diff</span>
              <Button size="sm" variant="ghost" onClick={loadGitInfo} className="text-[11px]">
                Refresh Status
              </Button>
            </div>
            <pre className="text-[11px] font-mono text-slate-300 bg-slate-950 p-3 rounded-lg overflow-x-auto max-h-56 leading-relaxed border border-slate-800/80">
              {gitDiff || '(Working tree bersih atau belum ada diff yang dapat ditampilkan)'}
            </pre>
          </Card>

          {/* Commit Dialog with Secret Scanning Guard */}
          <Card className="p-3 bg-slate-800/70 border-slate-700 space-y-2">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-semibold text-slate-200">Safe Commit (Protected by Secret Scanner)</h3>
              <Badge variant="success" className="text-[10px]">
                Pre-commit Hook: Active
              </Badge>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Pesan commit (misal: feat(browser): implement SSRF protection)"
                value={commitMsg}
                onChange={(e) => setCommitMsg(e.target.value)}
                className="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white"
              />
              <Button
                size="sm"
                onClick={handleCommit}
                disabled={committing || !commitMsg.trim()}
                className="text-xs"
              >
                {committing ? 'Memindai & Commit...' : 'Commit Perubahan'}
              </Button>
            </div>
          </Card>
        </div>
      )}

      {/* Tab 2: Sandboxed Command Terminal */}
      {activeTab === 'terminal' && (
        <div className="space-y-4">
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-slate-200">Allowlisted Binary Runner</span>
              <span className="text-[11px] text-slate-400">
                Didukung: git, cargo, pnpm, npm, node, python, uv, pytest, rustc, tsc, ruff, eslint
              </span>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Ketik perintah (contoh: cargo test --lib, pnpm build, git status)"
                value={commandStr}
                onChange={(e) => setCommandStr(e.target.value)}
                className="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-xs font-mono text-white"
              />
              <Button size="sm" onClick={handleRunCommand} disabled={runningCmd} className="text-xs">
                {runningCmd ? 'Menjalankan...' : 'Jalankan'}
              </Button>
            </div>
          </Card>

          {/* Terminal Console Output */}
          <Card className="p-3 bg-slate-950 border-slate-800 space-y-2">
            <div className="flex items-center justify-between text-xs text-slate-400 pb-1 border-b border-slate-800">
              <div className="flex items-center gap-2">
                <span className="w-2.5 h-2.5 rounded-full bg-slate-700"></span>
                <span className="font-mono">Console Output (Redacted)</span>
              </div>
              {terminalOutput && (
                <div className="flex items-center gap-3">
                  <span className="font-mono text-slate-400">Durasi: {terminalOutput.duration_ms} ms</span>
                  <Badge
                    variant={terminalOutput.exit_code === 0 ? 'success' : 'danger'}
                    className="text-[10px]"
                  >
                    Exit Code: {terminalOutput.exit_code}
                  </Badge>
                </div>
              )}
            </div>

            <pre className="text-xs font-mono text-slate-200 bg-black/60 p-3 rounded-lg overflow-x-auto min-h-[220px] max-h-[380px] leading-relaxed border border-slate-900 whitespace-pre-wrap">
              {terminalOutput
                ? (terminalOutput.stdout || '') + (terminalOutput.stderr ? `\n[STDERR]\n${terminalOutput.stderr}` : '')
                : 'Belum ada perintah yang dijalankan. Masukkan perintah allowlisted di atas.'}
            </pre>
          </Card>
        </div>
      )}

      {/* Tab 3: Software Engineering Standards (SKPL / DPPL) */}
      {activeTab === 'templates' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {templates.map((tpl) => (
              <Card
                key={tpl.id}
                onClick={() => setSelectedTemplate(tpl.id)}
                className={`p-3 cursor-pointer transition-all border ${
                  selectedTemplate === tpl.id
                    ? 'bg-indigo-950/40 border-indigo-500'
                    : 'bg-slate-800/40 border-slate-700 hover:border-slate-600'
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <Badge variant="violet" className="text-[10px]">
                    {tpl.standard}
                  </Badge>
                  <span className="text-[10px] text-slate-500 capitalize">{tpl.category}</span>
                </div>
                <h4 className="text-xs font-semibold text-white">{tpl.title}</h4>
                <p className="text-[11px] text-slate-400 mt-1 line-clamp-2">{tpl.description}</p>
              </Card>
            ))}
          </div>

          {/* Generator Controls */}
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-3">
            <h3 className="text-xs font-semibold text-slate-200">Konfigurasi & Generate Dokumen Standar</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Nama Proyek Perangkat Lunak</label>
                <input
                  type="text"
                  value={projectName}
                  onChange={(e) => setProjectName(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
                />
              </div>
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Lead Architect / Penulis</label>
                <input
                  type="text"
                  value={authorName}
                  onChange={(e) => setAuthorName(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
                />
              </div>
            </div>
            <div className="flex justify-end">
              <Button size="sm" onClick={handleRenderTemplate} disabled={loading} className="text-xs">
                {loading ? 'Membuat Dokumen...' : 'Render Dokumen Standar'}
              </Button>
            </div>
          </Card>

          {/* Rendered Markdown Output */}
          {renderedTemplate && (
            <Card className="p-4 bg-slate-900 border-slate-800 space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="text-xs font-semibold text-indigo-400">Output Standar Rekayasa Terstruktur</span>
                <Badge variant="success" className="text-[10px]">
                  Ready for Export
                </Badge>
              </div>
              <pre className="text-xs font-mono text-slate-300 bg-slate-950 p-3 rounded-lg overflow-x-auto max-h-96 whitespace-pre-wrap leading-relaxed border border-slate-800">
                {renderedTemplate}
              </pre>
            </Card>
          )}
        </div>
      )}
    </div>
  );
};

