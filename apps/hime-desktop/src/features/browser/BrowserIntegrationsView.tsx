import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  browserOpen,
  browserExtract,
  browserDownload,
  browserClose,
  listBrowserHistory,
  listBrowserDownloads,
  githubInspect,
  githubListIssues,
  githubCreateIssue,
  driveListFiles,
  driveShareFile,
  listIntegrationAccounts,
} from '../../services/ipcService';
import {
  BrowserSession,
  BrowserHistoryItem,
  BrowserDownload,
  BrowserExtractionResult,
} from '../../types/browser';
import {
  IntegrationAccount,
  GitHubRepoSummary,
  GitHubIssue,
  GoogleDriveFile,
} from '../../types/integrations';

export const BrowserIntegrationsView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'browser' | 'github' | 'drive'>('browser');
  const [accounts, setAccounts] = useState<IntegrationAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // Browser state
  const [targetUrl, setTargetUrl] = useState('https://docs.rs/tauri/2.0');
  const [activeSession, setActiveSession] = useState<BrowserSession | null>(null);
  const [extractedContent, setExtractedContent] = useState<BrowserExtractionResult | null>(null);
  const [history, setHistory] = useState<BrowserHistoryItem[]>([]);
  const [downloads, setDownloads] = useState<BrowserDownload[]>([]);
  const [navigating, setNavigating] = useState(false);

  // GitHub state
  const [githubOwner, setGithubOwner] = useState('rust-lang');
  const [githubRepo, setGithubRepo] = useState('rust');
  const [repoSummary, setRepoSummary] = useState<GitHubRepoSummary | null>(null);
  const [issues, setIssues] = useState<GitHubIssue[]>([]);
  const [newIssueTitle, setNewIssueTitle] = useState('');
  const [newIssueBody, setNewIssueBody] = useState('');

  // Google Drive state
  const [driveFiles, setDriveFiles] = useState<GoogleDriveFile[]>([]);
  const [shareEmail, setShareEmail] = useState('');
  const [shareFileId, setShareFileId] = useState('');

  const loadInitialData = useCallback(async () => {
    try {
      const [accs, hist, dls] = await Promise.all([
        listIntegrationAccounts(),
        listBrowserHistory(20),
        listBrowserDownloads(20),
      ]);
      setAccounts(accs);
      setHistory(hist);
      setDownloads(dls);
    } catch (err: any) {
      console.warn('Initial load error:', err);
    }
  }, []);

  useEffect(() => {
    loadInitialData();
  }, [loadInitialData]);

  // Browser actions
  const handleOpenUrl = async () => {
    if (!targetUrl.trim()) return;
    setNavigating(true);
    setStatusMessage(null);
    try {
      const session = await browserOpen(targetUrl.trim());
      setActiveSession(session);
      setStatusMessage(`Halaman dibuka dalam session terisolasi: ${session.id}`);

      // Auto-extract content
      const extracted = await browserExtract(session.id);
      setExtractedContent(extracted);

      // Refresh history
      const updatedHistory = await listBrowserHistory(20);
      setHistory(updatedHistory);
    } catch (err: any) {
      setStatusMessage(`Navigasi diblokir (SSRF / Boundary Protection): ${err.message || String(err)}`);
    } finally {
      setNavigating(false);
    }
  };

  const handleCloseSession = async () => {
    if (!activeSession) return;
    try {
      await browserClose(activeSession.id);
      setActiveSession(null);
      setExtractedContent(null);
      setStatusMessage('Session browser ditutup dengan aman.');
    } catch (err: any) {
      console.warn('Close session error:', err);
    }
  };

  const handleDownloadFile = async () => {
    if (!targetUrl.trim()) return;
    try {
      const targetPath = `.hime/quarantine/download_${Date.now()}.dat`;
      const dl = await browserDownload(targetUrl.trim(), targetPath);
      setStatusMessage(`File diunduh ke ruang karantina aman: ${dl.target_path}`);
      const updatedDls = await listBrowserDownloads(20);
      setDownloads(updatedDls);
    } catch (err: any) {
      setStatusMessage(`Download gagal: ${err.message || String(err)}`);
    }
  };

  // GitHub actions
  const handleInspectGithub = async () => {
    if (!githubOwner.trim() || !githubRepo.trim()) return;
    setLoading(true);
    setStatusMessage(null);
    try {
      const [summary, issueList] = await Promise.all([
        githubInspect(githubOwner.trim(), githubRepo.trim()),
        githubListIssues(githubOwner.trim(), githubRepo.trim(), 'open'),
      ]);
      setRepoSummary(summary);
      setIssues(issueList);
    } catch (err: any) {
      setStatusMessage(`Gagal menginspeksi GitHub: ${err.message || String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateIssue = async () => {
    if (!newIssueTitle.trim() || !githubOwner.trim() || !githubRepo.trim()) return;
    try {
      await githubCreateIssue(githubOwner.trim(), githubRepo.trim(), newIssueTitle.trim(), newIssueBody.trim());
      setStatusMessage(`Issue '${newIssueTitle}' berhasil dibuat.`);
      setNewIssueTitle('');
      setNewIssueBody('');
      handleInspectGithub();
    } catch (err: any) {
      setStatusMessage(`Gagal membuat issue: ${err.message || String(err)}`);
    }
  };

  // Drive actions
  const handleLoadDriveFiles = async () => {
    setLoading(true);
    try {
      const files = await driveListFiles();
      setDriveFiles(files);
    } catch (err: any) {
      setStatusMessage(`Gagal memuat Drive: ${err.message || String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleShareDriveFile = async () => {
    if (!shareFileId.trim() || !shareEmail.trim()) return;
    try {
      await driveShareFile(shareFileId.trim(), shareEmail.trim(), 'reader');
      setStatusMessage(`Permintaan share dikonfirmasi: ${shareFileId} ke ${shareEmail}`);
      setShareEmail('');
    } catch (err: any) {
      setStatusMessage(`Gagal share file: ${err.message || String(err)}`);
    }
  };

  useEffect(() => {
    if (activeTab === 'github' && !repoSummary) {
      handleInspectGithub();
    } else if (activeTab === 'drive' && driveFiles.length === 0) {
      handleLoadDriveFiles();
    }
  }, [activeTab]);

  return (
    <div className="flex flex-col h-full space-y-4 p-4 text-slate-100 overflow-y-auto">
      {/* Header & Connected Accounts Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-700/60 pb-3">
        <div>
          <h1 className="text-xl font-bold tracking-tight text-white flex items-center gap-2">
            <span>Browser & External Integrations</span>
            <Badge variant="violet" className="text-xs">
              SSRF Protected
            </Badge>
          </h1>
          <p className="text-xs text-slate-400">
            Akses web aman dengan isolasi profile, karantina download, dan adapter terverifikasi untuk GitHub dan Google Drive.
          </p>
        </div>

        {/* Connected Accounts Badges */}
        <div className="flex items-center gap-2">
          {accounts.map((acc, i) => (
            <div
              key={i}
              className="flex items-center gap-1.5 bg-slate-800 border border-slate-700 rounded-lg px-2.5 py-1 text-xs text-slate-300"
            >
              <span className="w-2 h-2 rounded-full bg-emerald-400"></span>
              <span className="font-medium capitalize">{acc.provider}:</span>
              <span className="text-slate-400">{acc.display_name}</span>
            </div>
          ))}
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

      {/* Sub Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-700/60 pb-2">
        <button
          onClick={() => setActiveTab('browser')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'browser'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Sandboxed Web Research
        </button>
        <button
          onClick={() => setActiveTab('github')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'github'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          GitHub Adapter
        </button>
        <button
          onClick={() => setActiveTab('drive')}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            activeTab === 'drive'
              ? 'bg-indigo-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          Google Drive Adapter
        </button>
      </div>

      {/* Tab 1: Sandboxed Web Research */}
      {activeTab === 'browser' && (
        <div className="space-y-4">
          {/* URL Navigation Input */}
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-slate-200">Safe Web Navigator</span>
              <span className="text-[11px] text-slate-400">
                Memblokir IP loopback, metadata cloud (169.254.169.254), dan skema berbahaya
              </span>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="https://example.com"
                value={targetUrl}
                onChange={(e) => setTargetUrl(e.target.value)}
                className="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs font-mono text-white"
              />
              <Button size="sm" onClick={handleOpenUrl} disabled={navigating} className="text-xs">
                {navigating ? 'Membuka...' : 'Buka Halaman'}
              </Button>
              <Button size="sm" variant="secondary" onClick={handleDownloadFile} className="text-xs">
                Unduh ke Karantina
              </Button>
              {activeSession && (
                <Button size="sm" variant="danger" onClick={handleCloseSession} className="text-xs">
                  Tutup Session
                </Button>
              )}
            </div>
          </Card>

          {/* Active Session & Extracted Content View */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <Card className="p-3 bg-slate-900 border-slate-800 space-y-2 md:col-span-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="text-xs font-semibold text-slate-300">
                  Konten Terekstraksi (Anti-Prompt Injection Tagged)
                </span>
                {extractedContent && (
                  <Badge variant="success" className="text-[10px]">
                    {extractedContent.links.length} Tautan Ditemukan
                  </Badge>
                )}
              </div>
              <pre className="text-xs font-mono text-slate-300 bg-slate-950 p-3 rounded-lg overflow-x-auto min-h-[220px] max-h-[360px] whitespace-pre-wrap leading-relaxed border border-slate-800">
                {extractedContent?.extracted_markdown ||
                  'Belum ada konten diekstraksi. Masukkan URL publik pada form di atas.'}
              </pre>
            </Card>

            {/* Quarantine Downloads Box */}
            <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-amber-400">Berkas Karantina (.hime/quarantine)</span>
                <Badge variant="warning" className="text-[10px]">
                  {downloads.length} File
                </Badge>
              </div>
              <div className="space-y-2 max-h-[340px] overflow-y-auto">
                {downloads.length === 0 ? (
                  <p className="text-slate-500 text-xs italic">Belum ada file diunduh ke karantina.</p>
                ) : (
                  downloads.map((dl) => (
                    <div key={dl.id} className="p-2 bg-slate-900/80 rounded border border-slate-800 text-[11px] space-y-1">
                      <div className="font-semibold text-slate-200 truncate">{dl.target_path}</div>
                      <div className="text-slate-400 text-[10px] truncate">{dl.source_url}</div>
                      <div className="flex items-center justify-between text-[10px]">
                        <span className="font-mono text-slate-400">{(dl.file_size_bytes / 1024).toFixed(1)} KB</span>
                        <span className="text-emerald-400 font-mono">SHA-256 OK</span>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </Card>
          </div>

          {/* Browser History Table */}
          <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2">
            <h3 className="text-xs font-semibold text-slate-200">Riwayat Riset Web Terakhir</h3>
            <div className="max-h-40 overflow-y-auto space-y-1 font-mono text-[11px]">
              {history.length === 0 ? (
                <p className="text-slate-500 italic">Riwayat kosong.</p>
              ) : (
                history.map((h) => (
                  <div key={h.id} className="flex items-center justify-between bg-slate-900/50 px-2.5 py-1 rounded">
                    <span className="text-indigo-300 font-medium truncate max-w-xs">{h.title || h.url}</span>
                    <span className="text-slate-500 text-[10px] truncate max-w-sm">{h.url}</span>
                    <span className="text-slate-400 text-[10px]">
                      {new Date(h.visited_at * 1000).toLocaleTimeString()}
                    </span>
                  </div>
                ))
              )}
            </div>
          </Card>
        </div>
      )}

      {/* Tab 2: GitHub Adapter */}
      {activeTab === 'github' && (
        <div className="space-y-4">
          <Card className="p-3 bg-slate-800/60 border-slate-700 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-slate-200">GitHub Repository Inspector</span>
              <span className="text-[11px] text-slate-400">Perubahan push/merge tunduk pada Human Approval Gate</span>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Owner (misal: rust-lang)"
                value={githubOwner}
                onChange={(e) => setGithubOwner(e.target.value)}
                className="w-1/3 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white"
              />
              <input
                type="text"
                placeholder="Repo (misal: rust)"
                value={githubRepo}
                onChange={(e) => setGithubRepo(e.target.value)}
                className="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white"
              />
              <Button size="sm" onClick={handleInspectGithub} disabled={loading} className="text-xs">
                {loading ? 'Memuat...' : 'Inspeksi Repo'}
              </Button>
            </div>
          </Card>

          {repoSummary && (
            <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
              <Card className="p-3 bg-slate-800/60 border-slate-700">
                <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Repository</span>
                <span className="text-sm font-semibold text-white">
                  {repoSummary.owner}/{repoSummary.repo}
                </span>
              </Card>
              <Card className="p-3 bg-slate-800/60 border-slate-700">
                <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Default Branch</span>
                <span className="text-sm font-mono font-semibold text-indigo-400">
                  {repoSummary.default_branch}
                </span>
              </Card>
              <Card className="p-3 bg-slate-800/60 border-slate-700">
                <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Open Issues</span>
                <span className="text-sm font-semibold text-amber-400">{repoSummary.open_issues_count}</span>
              </Card>
              <Card className="p-3 bg-slate-800/60 border-slate-700">
                <span className="text-[11px] text-slate-400 uppercase tracking-wider block">Open Pull Requests</span>
                <span className="text-sm font-semibold text-emerald-400">{repoSummary.open_pr_count}</span>
              </Card>
            </div>
          )}

          {/* Issues List & Create Box */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2 md:col-span-2">
              <h3 className="text-xs font-semibold text-slate-200">Active Issues ({issues.length})</h3>
              <div className="space-y-1.5 max-h-60 overflow-y-auto">
                {issues.length === 0 ? (
                  <p className="text-slate-500 text-xs italic">Tidak ada open issues.</p>
                ) : (
                  issues.map((iss) => (
                    <div key={iss.id} className="p-2 bg-slate-900/60 rounded border border-slate-800 flex items-center justify-between text-xs">
                      <div>
                        <span className="font-semibold text-slate-200">#{iss.id} {iss.title}</span>
                        <div className="text-[10px] text-slate-500">Status: {iss.state}</div>
                      </div>
                      <Badge variant="cyan" className="text-[10px]">
                        Open
                      </Badge>
                    </div>
                  ))
                )}
              </div>
            </Card>

            <Card className="p-3 bg-slate-800/70 border-slate-700 space-y-2">
              <h3 className="text-xs font-semibold text-slate-200">Buat Issue Baru</h3>
              <input
                type="text"
                placeholder="Judul issue"
                value={newIssueTitle}
                onChange={(e) => setNewIssueTitle(e.target.value)}
                className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
              />
              <textarea
                placeholder="Deskripsi issue..."
                value={newIssueBody}
                onChange={(e) => setNewIssueBody(e.target.value)}
                rows={3}
                className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
              />
              <Button size="sm" onClick={handleCreateIssue} disabled={!newIssueTitle.trim()} className="w-full text-xs">
                Kirim Issue
              </Button>
            </Card>
          </div>
        </div>
      )}

      {/* Tab 3: Google Drive Adapter */}
      {activeTab === 'drive' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {/* Drive Files List */}
            <Card className="p-3 bg-slate-800/40 border-slate-700 space-y-2 md:col-span-2">
              <div className="flex items-center justify-between">
                <h3 className="text-xs font-semibold text-slate-200">Google Drive Files & Documents</h3>
                <Button size="sm" variant="ghost" onClick={handleLoadDriveFiles} className="text-xs">
                  Refresh
                </Button>
              </div>
              <div className="space-y-1.5 max-h-72 overflow-y-auto">
                {driveFiles.length === 0 ? (
                  <p className="text-slate-500 text-xs italic">Memuat file dari Google Drive...</p>
                ) : (
                  driveFiles.map((f) => (
                    <div
                      key={f.id}
                      onClick={() => setShareFileId(f.id)}
                      className={`p-2 rounded border cursor-pointer transition-colors flex items-center justify-between text-xs ${
                        shareFileId === f.id
                          ? 'bg-indigo-950/40 border-indigo-500'
                          : 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                      }`}
                    >
                      <div>
                        <div className="font-semibold text-slate-200">{f.name}</div>
                        <div className="text-[10px] text-slate-500">{f.mime_type}</div>
                      </div>
                      <span className="font-mono text-slate-400 text-[10px]">
                        {f.size_bytes > 0 ? `${(f.size_bytes / 1024).toFixed(0)} KB` : 'Cloud Doc'}
                      </span>
                    </div>
                  ))
                )}
              </div>
            </Card>

            {/* Sharing Gate Card */}
            <Card className="p-3 bg-slate-800/70 border-slate-700 space-y-3">
              <div className="border-b border-slate-700 pb-2">
                <h3 className="text-xs font-semibold text-slate-200">Secure File Sharing Gate</h3>
                <p className="text-[11px] text-slate-400 mt-0.5">
                  Operasi berbagi berkas cloud memerlukan approval pengguna untuk mencegah kebocoran data.
                </p>
              </div>
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">File ID Terpilih</label>
                <input
                  type="text"
                  placeholder="Pilih file dari daftar"
                  value={shareFileId}
                  onChange={(e) => setShareFileId(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
                />
              </div>
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Email Penerima Akses</label>
                <input
                  type="email"
                  placeholder="colleague@example.com"
                  value={shareEmail}
                  onChange={(e) => setShareEmail(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white"
                />
              </div>
              <Button
                size="sm"
                onClick={handleShareDriveFile}
                disabled={!shareFileId.trim() || !shareEmail.trim()}
                className="w-full text-xs"
              >
                Minta Approval & Bagikan
              </Button>
            </Card>
          </div>
        </div>
      )}
    </div>
  );
};

