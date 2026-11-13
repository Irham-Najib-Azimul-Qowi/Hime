import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { Switch } from '../../components/ui/Switch';
import {
  getStorageAnalysis,
  getStorageDuplicates,
  getStorageOperations,
  restoreStorageOperation,
  getStorageRules,
  toggleStorageRule,
  searchStorageFiles,
  scanStorageDirectory,
} from '../../services/ipcService';
import {
  StorageAnalysisReport,
  DuplicateGroup,
  FileOperation,
  FileRule,
  IndexedFile,
} from '../../types/storage';

export const StorageView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'analytics' | 'duplicates' | 'operations' | 'rules'>('analytics');
  const [loading, setLoading] = useState(false);
  const [analysis, setAnalysis] = useState<StorageAnalysisReport | null>(null);
  const [duplicates, setDuplicates] = useState<DuplicateGroup[]>([]);
  const [operations, setOperations] = useState<FileOperation[]>([]);
  const [rules, setRules] = useState<FileRule[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<IndexedFile[]>([]);
  const [scanPath, setScanPath] = useState('');
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [anaRes, dupRes, opsRes, rulesRes] = await Promise.all([
        getStorageAnalysis(10),
        getStorageDuplicates(),
        getStorageOperations(50),
        getStorageRules(),
      ]);
      setAnalysis(anaRes);
      setDuplicates(dupRes);
      setOperations(opsRes);
      setRules(rulesRes);
    } catch (err: any) {
      setStatusMessage(`Error loading storage data: ${err?.message || err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      const res = await searchStorageFiles(searchQuery, 25);
      setSearchResults(res);
    } catch (err: any) {
      setStatusMessage(`Search failed: ${err?.message || err}`);
    }
  };

  const handleScan = async () => {
    if (!scanPath.trim()) return;
    setStatusMessage(`Scanning ${scanPath}...`);
    try {
      const res = await scanStorageDirectory(scanPath);
      setStatusMessage(`Scan completed: indexed ${res.indexed_count} files/directories.`);
      loadData();
    } catch (err: any) {
      setStatusMessage(`Scan failed: ${err?.message || err}`);
    }
  };

  const handleRestore = async (originalOpId: string) => {
    try {
      await restoreStorageOperation(originalOpId);
      setStatusMessage(`Operation ${originalOpId} successfully restored from Hime Recycle.`);
      loadData();
    } catch (err: any) {
      setStatusMessage(`Restore failed: ${err?.message || err}`);
    }
  };

  const handleToggleRule = async (ruleId: string, currentStatus: boolean) => {
    try {
      await toggleStorageRule(ruleId, !currentStatus);
      loadData();
    } catch (err: any) {
      setStatusMessage(`Rule toggle failed: ${err?.message || err}`);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (epochSecs: number) => {
    if (!epochSecs) return '-';
    return new Date(epochSecs * 1000).toLocaleString();
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', height: '100%' }}>
      {/* Top Banner / Stat Cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '16px' }}>
        <Card>
          <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>Indexed Files</div>
          <div style={{ fontSize: '28px', fontWeight: 600, color: 'var(--color-text-primary)', marginTop: '4px' }}>
            {analysis?.base_report.total_files.toLocaleString() ?? 0}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--color-text-muted)', marginTop: '4px' }}>
            Managed by Hime Local Storage Engine
          </div>
        </Card>

        <Card>
          <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>Total Storage Analyzed</div>
          <div style={{ fontSize: '28px', fontWeight: 600, color: 'var(--color-primary)', marginTop: '4px' }}>
            {formatBytes(analysis?.base_report.total_bytes ?? 0)}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--color-text-muted)', marginTop: '4px' }}>
            Across user-allowed local roots
          </div>
        </Card>

        <Card>
          <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>Potential Cleanable</div>
          <div style={{ fontSize: '28px', fontWeight: 600, color: '#f59e0b', marginTop: '4px' }}>
            {formatBytes(analysis?.cleanable_bytes ?? 0)}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--color-text-muted)', marginTop: '4px' }}>
            Old downloads, installers & cache files
          </div>
        </Card>

        <Card>
          <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>Duplicate Groups</div>
          <div style={{ fontSize: '28px', fontWeight: 600, color: 'var(--color-text-primary)', marginTop: '4px' }}>
            {duplicates.length}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--color-text-muted)', marginTop: '4px' }}>
            Verified identical content SHA-256
          </div>
        </Card>
      </div>

      {/* Directory Scanner & Search Bar */}
      <Card>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', alignItems: 'center', justifyContent: 'space-between' }}>
          <form onSubmit={handleSearch} style={{ display: 'flex', gap: '8px', flex: '1 1 300px' }}>
            <input
              type="text"
              placeholder="Search indexed files by keyword or extension..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{
                flex: 1,
                padding: '8px 12px',
                borderRadius: '8px',
                border: '1px solid var(--color-border)',
                background: 'var(--color-surface-hover)',
                color: 'var(--color-text-primary)',
                fontSize: '13px',
              }}
            />
            <Button type="submit" variant="primary">Search</Button>
          </form>

          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            <input
              type="text"
              placeholder="Folder path to index (e.g. C:\Users\...)"
              value={scanPath}
              onChange={(e) => setScanPath(e.target.value)}
              style={{
                padding: '8px 12px',
                borderRadius: '8px',
                border: '1px solid var(--color-border)',
                background: 'var(--color-surface-hover)',
                color: 'var(--color-text-primary)',
                fontSize: '13px',
                width: '260px',
              }}
            />
            <Button onClick={handleScan} variant="secondary">Scan Folder</Button>
            <Button onClick={loadData} variant="ghost" disabled={loading}>Refresh</Button>
          </div>
        </div>

        {statusMessage && (
          <div style={{ marginTop: '10px', fontSize: '12px', color: 'var(--color-primary)', background: 'var(--color-primary-light)', padding: '6px 12px', borderRadius: '6px' }}>
            {statusMessage}
          </div>
        )}

        {/* Search Results Display */}
        {searchResults.length > 0 && (
          <div style={{ marginTop: '14px', borderTop: '1px solid var(--color-border)', paddingTop: '10px' }}>
            <div style={{ fontSize: '12px', fontWeight: 600, marginBottom: '8px' }}>
              Search Results ({searchResults.length}):
            </div>
            <div style={{ maxHeight: '160px', overflowY: 'auto' }}>
              {searchResults.map((f, idx) => (
                <div key={idx} style={{ display: 'flex', justifyContent: 'space-between', padding: '6px 0', borderBottom: '1px solid var(--color-border-subtle)', fontSize: '12px' }}>
                  <span style={{ fontFamily: 'monospace' }}>{f.path}</span>
                  <span style={{ color: 'var(--color-text-muted)' }}>{formatBytes(f.size_bytes)}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </Card>

      {/* Sub-tabs Navigation */}
      <div style={{ display: 'flex', gap: '8px', borderBottom: '1px solid var(--color-border)', paddingBottom: '4px' }}>
        <button
          onClick={() => setActiveTab('analytics')}
          style={{
            padding: '8px 16px',
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            borderBottom: activeTab === 'analytics' ? '2px solid var(--color-primary)' : '2px solid transparent',
            color: activeTab === 'analytics' ? 'var(--color-primary)' : 'var(--color-text-secondary)',
            fontWeight: activeTab === 'analytics' ? 600 : 400,
          }}
        >
          Analytics & Breakdown
        </button>
        <button
          onClick={() => setActiveTab('duplicates')}
          style={{
            padding: '8px 16px',
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            borderBottom: activeTab === 'duplicates' ? '2px solid var(--color-primary)' : '2px solid transparent',
            color: activeTab === 'duplicates' ? 'var(--color-primary)' : 'var(--color-text-secondary)',
            fontWeight: activeTab === 'duplicates' ? 600 : 400,
          }}
        >
          Duplicates & Versions
        </button>
        <button
          onClick={() => setActiveTab('operations')}
          style={{
            padding: '8px 16px',
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            borderBottom: activeTab === 'operations' ? '2px solid var(--color-primary)' : '2px solid transparent',
            color: activeTab === 'operations' ? 'var(--color-primary)' : 'var(--color-text-secondary)',
            fontWeight: activeTab === 'operations' ? 600 : 400,
          }}
        >
          Journal & Hime Recycle
        </button>
        <button
          onClick={() => setActiveTab('rules')}
          style={{
            padding: '8px 16px',
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            borderBottom: activeTab === 'rules' ? '2px solid var(--color-primary)' : '2px solid transparent',
            color: activeTab === 'rules' ? 'var(--color-primary)' : 'var(--color-text-secondary)',
            fontWeight: activeTab === 'rules' ? 600 : 400,
          }}
        >
          Smart Organizer Rules
        </button>
      </div>

      {/* Tab 1: Analytics & Breakdown */}
      {activeTab === 'analytics' && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))', gap: '16px' }}>
          <Card title="Disk Usage by Extension">
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {analysis?.base_report.by_extension.map((ext, idx) => (
                <div key={idx} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '13px' }}>
                  <span style={{ fontWeight: 500 }}>.{ext.extension || 'none'}</span>
                  <div style={{ display: 'flex', gap: '16px', alignItems: 'center' }}>
                    <span style={{ color: 'var(--color-text-muted)' }}>{ext.count} files</span>
                    <Badge variant="default">{formatBytes(ext.total_bytes)}</Badge>
                  </div>
                </div>
              ))}
            </div>
          </Card>

          <Card title="Cleanable Suggestions">
            {analysis?.cleanable_suggestions.length === 0 ? (
              <div style={{ fontSize: '13px', color: 'var(--color-text-muted)' }}>No cleanable files detected.</div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                {analysis?.cleanable_suggestions.map((s, idx) => (
                  <div key={idx} style={{ padding: '8px', borderRadius: '6px', background: 'var(--color-surface-hover)', fontSize: '12px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', fontWeight: 600 }}>
                      <span>{s.category}</span>
                      <span style={{ color: '#f59e0b' }}>{formatBytes(s.size_bytes)}</span>
                    </div>
                    <div style={{ color: 'var(--color-text-secondary)', marginTop: '2px', wordBreak: 'break-all' }}>{s.path}</div>
                    <div style={{ color: 'var(--color-text-muted)', fontSize: '11px', marginTop: '2px' }}>{s.reason}</div>
                  </div>
                ))}
              </div>
            )}
          </Card>

          <Card title="Top Largest Files" style={{ gridColumn: '1 / -1' }}>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {analysis?.base_report.largest_files.map((file, idx) => (
                <div key={idx} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '6px 0', borderBottom: '1px solid var(--color-border-subtle)', fontSize: '12px' }}>
                  <div style={{ display: 'flex', flexDirection: 'column' }}>
                    <span style={{ fontWeight: 500 }}>{file.file_name}</span>
                    <span style={{ color: 'var(--color-text-muted)', fontSize: '11px' }}>{file.parent_dir}</span>
                  </div>
                  <div style={{ display: 'flex', gap: '16px', alignItems: 'center' }}>
                    <span style={{ color: 'var(--color-text-muted)' }}>{formatDate(file.modified_at)}</span>
                    <Badge variant="violet">{formatBytes(file.size_bytes)}</Badge>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        </div>
      )}

      {/* Tab 2: Duplicates & Versions */}
      {activeTab === 'duplicates' && (
        <Card title="Exact Duplicates (Identical SHA-256)">
          {duplicates.length === 0 ? (
            <div style={{ fontSize: '13px', color: 'var(--color-text-muted)' }}>No duplicate files detected.</div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
              {duplicates.map((grp, idx) => (
                <div key={idx} style={{ padding: '12px', borderRadius: '8px', border: '1px solid var(--color-border)', background: 'var(--color-surface-hover)' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                    <span style={{ fontSize: '12px', fontFamily: 'monospace', color: 'var(--color-text-secondary)' }}>
                      Hash: {grp.content_hash.slice(0, 16)}...
                    </span>
                    <Badge variant="warning">{formatBytes(grp.size_bytes)} each</Badge>
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                    {grp.files.map((f, fIdx) => (
                      <div key={fIdx} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', padding: '4px 8px', background: 'var(--color-surface)', borderRadius: '4px' }}>
                        <span>{f.path}</span>
                        <span style={{ color: 'var(--color-text-muted)' }}>Modified: {formatDate(f.modified_at)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* Tab 3: Safe Operations Journal & Hime Recycle */}
      {activeTab === 'operations' && (
        <Card title="Safe File Operations Journal & One-Click Restore">
          <div style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginBottom: '12px' }}>
            All destructive or move operations are journaled and backed up into Hime Recycle before modification.
          </div>
          {operations.length === 0 ? (
            <div style={{ fontSize: '13px', color: 'var(--color-text-muted)' }}>No file operations recorded in journal.</div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
              {operations.map((op) => (
                <div key={op.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '10px 12px', borderRadius: '8px', border: '1px solid var(--color-border)', background: 'var(--color-surface)' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', flex: 1 }}>
                    <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                      <Badge variant={op.status === 'committed' ? 'success' : op.status === 'rolled_back' ? 'warning' : 'default'}>
                        {op.op_type.toUpperCase()} ({op.status})
                      </Badge>
                      <span style={{ fontSize: '12px', fontWeight: 600 }}>{op.id}</span>
                      <span style={{ fontSize: '11px', color: 'var(--color-text-muted)' }}>by {op.actor} at {formatDate(op.started_at)}</span>
                    </div>
                    <div style={{ fontSize: '12px', marginTop: '4px' }}>
                      <span style={{ color: 'var(--color-text-secondary)' }}>Source: </span>
                      <span style={{ fontFamily: 'monospace' }}>{op.source_path}</span>
                    </div>
                    {op.destination_path && (
                      <div style={{ fontSize: '12px' }}>
                        <span style={{ color: 'var(--color-text-secondary)' }}>Destination: </span>
                        <span style={{ fontFamily: 'monospace' }}>{op.destination_path}</span>
                      </div>
                    )}
                    {op.backup_path && (
                      <div style={{ fontSize: '11px', color: 'var(--color-text-muted)' }}>
                        Recycle Location: {op.backup_path}
                      </div>
                    )}
                  </div>

                  {op.backup_path && op.status === 'committed' && (
                    <Button onClick={() => handleRestore(op.id)} variant="secondary" size="sm">
                      1-Click Restore
                    </Button>
                  )}
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* Tab 4: Smart Organizer Rules */}
      {activeTab === 'rules' && (
        <Card title="Active Smart Organization Rules">
          <div style={{ fontSize: '12px', color: 'var(--color-text-muted)', marginBottom: '12px' }}>
            Rules automatically propose categorization and file moves when matching downloaded or created documents.
          </div>
          {rules.length === 0 ? (
            <div style={{ fontSize: '13px', color: 'var(--color-text-muted)' }}>No rules defined.</div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
              {rules.map((rule) => (
                <div key={rule.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '12px', borderRadius: '8px', border: '1px solid var(--color-border)', background: 'var(--color-surface)' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
                    <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                      <span style={{ fontSize: '14px', fontWeight: 600 }}>{rule.name}</span>
                      <Badge variant="violet">{rule.pattern}</Badge>
                    </div>
                    <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginTop: '4px' }}>
                      Target: <span style={{ fontFamily: 'monospace' }}>{rule.target_dir}</span>
                    </div>
                    {rule.naming_template && (
                      <div style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>
                        Naming Template: <span style={{ fontFamily: 'monospace' }}>{rule.naming_template}</span>
                      </div>
                    )}
                  </div>
                  <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
                    <Switch
                      checked={rule.is_active}
                      onChange={() => handleToggleRule(rule.id, rule.is_active)}
                    />
                    <span style={{ fontSize: '12px', color: rule.is_active ? 'var(--color-success)' : 'var(--color-text-muted)' }}>
                      {rule.is_active ? 'Active' : 'Disabled'}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}
    </div>
  );
};
