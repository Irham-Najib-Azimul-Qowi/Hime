import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  getKanbanBoard,
  getKanbanColumns,
  listProductivityTasks,
  createProductivityTask,
  moveProductivityTask,
  parseNaturalTask,
  listStudySchedules,
  planStudySchedule,
  getAccountabilitySettings,
  setAccountabilityMode,
  getAccountabilityCheckin,
} from '../../services/ipcService';
import {
  KanbanBoard,
  KanbanColumn,
  ProductivityTask,
  StudySchedule,
  AccountabilitySettings,
  StartupCheckinSummary,
} from '../../types/productivity';

export const ProductivityView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'kanban' | 'study' | 'accountability'>('kanban');
  const [board, setBoard] = useState<KanbanBoard | null>(null);
  const [columns, setColumns] = useState<KanbanColumn[]>([]);
  const [tasks, setTasks] = useState<ProductivityTask[]>([]);
  const [schedules, setSchedules] = useState<StudySchedule[]>([]);
  const [accountability, setAccountability] = useState<AccountabilitySettings | null>(null);
  const [checkin, setCheckin] = useState<StartupCheckinSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // Quick task input state
  const [quickInput, setQuickInput] = useState('');
  const [isParsing, setIsParsing] = useState(false);

  // Study plan form state
  const [studyTitle, setStudyTitle] = useState('');
  const [studySubject, setStudySubject] = useState('');
  const [studyDuration, setStudyDuration] = useState('45');
  const [studyPriority, setStudyPriority] = useState('medium');

  const loadAllData = useCallback(async () => {
    setLoading(true);
    try {
      const [boardRes, colsRes, tasksRes, schedsRes, accRes, checkinRes] = await Promise.all([
        getKanbanBoard(),
        getKanbanColumns(),
        listProductivityTasks(),
        listStudySchedules(),
        getAccountabilitySettings(),
        getAccountabilityCheckin(),
      ]);
      setBoard(boardRes);
      setColumns(colsRes);
      setTasks(tasksRes);
      setSchedules(schedsRes);
      setAccountability(accRes);
      setCheckin(checkinRes);
    } catch (err: any) {
      setStatusMessage(`Error loading productivity data: ${err?.message || err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAllData();
  }, [loadAllData]);

  const handleQuickAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!quickInput.trim()) return;

    setIsParsing(true);
    try {
      const parsed = await parseNaturalTask(quickInput);
      const targetCol = columns[0]?.id || 'col-todo';
      const targetBoard = board?.id || 'board-default';

      await createProductivityTask(
        targetBoard,
        targetCol,
        parsed.title || quickInput,
        undefined,
        parsed.priority || 'medium',
        parsed.due_date,
        parsed.tags?.join(',') || 'quick-add'
      );
      setQuickInput('');
      setStatusMessage(`Task created: "${parsed.title || quickInput}"`);
      await loadAllData();
    } catch (err: any) {
      setStatusMessage(`Failed to create task: ${err?.message || err}`);
    } finally {
      setIsParsing(false);
    }
  };

  const handleMoveTask = async (taskId: string, targetColId: string) => {
    try {
      await moveProductivityTask(taskId, targetColId);
      await loadAllData();
    } catch (err: any) {
      setStatusMessage(`Failed to move task: ${err?.message || err}`);
    }
  };

  const handleCreateStudyBlock = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!studyTitle.trim() || !studySubject.trim()) return;

    try {
      const startTime = Math.floor(Date.now() / 1000) + 1800; // start 30m from now
      await planStudySchedule(
        studyTitle,
        studySubject,
        startTime,
        parseInt(studyDuration, 10) || 45,
        studyPriority
      );
      setStudyTitle('');
      setStudySubject('');
      setStatusMessage(`Study block "${studyTitle}" scheduled with 10-minute collision buffer.`);
      await loadAllData();
    } catch (err: any) {
      setStatusMessage(`Failed to schedule study block: ${err?.message || err}`);
    }
  };

  const handleSetMode = async (mode: 'santai' | 'biasa' | 'strict') => {
    try {
      await setAccountabilityMode(mode, accountability?.opt_in_screen_time || false);
      setStatusMessage(`Accountability mode updated to "${mode.toUpperCase()}".`);
      await loadAllData();
    } catch (err: any) {
      setStatusMessage(`Failed to update mode: ${err?.message || err}`);
    }
  };

  const handleToggleScreenTimeOptIn = async () => {
    if (!accountability) return;
    const newVal = !accountability.opt_in_screen_time;
    try {
      await setAccountabilityMode(accountability.mode, newVal);
      setStatusMessage(`Screen time opt-in updated: ${newVal ? 'Active' : 'Disabled'}.`);
      await loadAllData();
    } catch (err: any) {
      setStatusMessage(`Failed to toggle screen time: ${err?.message || err}`);
    }
  };

  const priorityColor = (p: string): 'default' | 'success' | 'warning' | 'danger' | 'cyan' | 'violet' => {
    switch (p.toLowerCase()) {
      case 'urgent':
        return 'danger';
      case 'high':
        return 'warning';
      case 'medium':
        return 'cyan';
      default:
        return 'default';
    }
  };

  return (
    <div className="flex flex-col h-full overflow-y-auto p-6 space-y-6 text-obsidian-100">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-obsidian-800 pb-4">
        <div>
          <div className="flex items-center space-x-2">
            <span className="text-xs uppercase tracking-widest text-violet-400 font-mono">
              Productivity & Accountability (UR-10 / UR-11 / UR-15)
            </span>
          </div>
          <h1 className="text-xl font-semibold text-white mt-1">Productivity & Task Management</h1>
          <p className="text-xs text-obsidian-400 mt-0.5">
            Deterministic Kanban, collision-free study planner, and non-punitive accountability ("Hime menagih, bukan mengunci").
          </p>
        </div>

        <div className="flex items-center space-x-3">
          {accountability && (
            <Badge
              variant={
                accountability.mode === 'strict'
                  ? 'warning'
                  : accountability.mode === 'biasa'
                  ? 'cyan'
                  : 'default'
              }
            >
              Mode: {accountability.mode.toUpperCase()}
            </Badge>
          )}
          <Button variant="secondary" size="sm" onClick={loadAllData} disabled={loading}>
            {loading ? 'Refreshing...' : 'Refresh'}
          </Button>
        </div>
      </div>

      {/* Status Alert */}
      {statusMessage && (
        <div className="bg-obsidian-800/80 border border-violet-500/30 rounded-lg p-3 text-xs text-violet-200 flex items-center justify-between">
          <span>{statusMessage}</span>
          <button
            onClick={() => setStatusMessage(null)}
            className="text-obsidian-400 hover:text-white ml-4"
          >
            ✕
          </button>
        </div>
      )}

      {/* Tabs */}
      <div className="flex space-x-2 border-b border-obsidian-800 pb-2">
        <button
          onClick={() => setActiveTab('kanban')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'kanban'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          Kanban Board
        </button>
        <button
          onClick={() => setActiveTab('study')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'study'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          Study Timetable
        </button>
        <button
          onClick={() => setActiveTab('accountability')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'accountability'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          Accountability & Ketegasan
        </button>
      </div>

      {/* TAB 1: KANBAN */}
      {activeTab === 'kanban' && (
        <div className="space-y-4">
          {/* Quick natural-language input */}
          <Card variant="surface-2">
            <form onSubmit={handleQuickAdd} className="flex gap-2">
              <input
                type="text"
                placeholder="Natural task parser (contoh: 'Selesaikan laporan bab 3 besok sore prioritas tinggi')..."
                value={quickInput}
                onChange={(e) => setQuickInput(e.target.value)}
                disabled={isParsing}
                className="flex-1 bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white placeholder-obsidian-500 focus:outline-none focus:border-violet-500 font-sans"
              />
              <Button type="submit" variant="primary" size="sm" disabled={isParsing || !quickInput.trim()}>
                {isParsing ? 'Parsing...' : 'Add Task'}
              </Button>
            </form>
          </Card>

          {/* Kanban Columns */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            {columns.map((col, colIdx) => {
              const colTasks = tasks.filter((t) => t.column_id === col.id);
              return (
                <div
                  key={col.id}
                  className="bg-obsidian-900/60 border border-obsidian-800 rounded-lg p-3 flex flex-col min-h-[450px]"
                >
                  <div className="flex items-center justify-between pb-3 mb-2 border-b border-obsidian-800/80">
                    <span className="text-xs font-semibold uppercase tracking-wider text-obsidian-300">
                      {col.name}
                    </span>
                    <span className="text-xs font-mono bg-obsidian-800 px-1.5 py-0.5 rounded text-obsidian-400">
                      {colTasks.length}
                    </span>
                  </div>

                  <div className="space-y-2.5 flex-1 overflow-y-auto">
                    {colTasks.length === 0 ? (
                      <div className="text-center py-8 text-xs text-obsidian-600 italic">
                        Empty column
                      </div>
                    ) : (
                      colTasks.map((t) => (
                        <div
                          key={t.id}
                          className="bg-obsidian-950/80 border border-obsidian-800 hover:border-obsidian-700 rounded-lg p-3 space-y-2 transition-all shadow-sm"
                        >
                          <div className="flex items-start justify-between gap-1">
                            <span className="text-xs font-medium text-white leading-snug line-clamp-2">
                              {t.title}
                            </span>
                            <Badge variant={priorityColor(t.priority)} size="sm">
                              {t.priority}
                            </Badge>
                          </div>

                          {t.description && (
                            <p className="text-[11px] text-obsidian-400 line-clamp-2">
                              {t.description}
                            </p>
                          )}

                          {t.due_date && (
                            <div className="text-[10px] text-violet-400 font-mono">
                              Deadline: {new Date(t.due_date * 1000).toLocaleDateString()}
                            </div>
                          )}

                          {/* Quick Move Buttons */}
                          <div className="flex items-center justify-between pt-2 border-t border-obsidian-800/60">
                            <button
                              onClick={() => {
                                if (colIdx > 0) handleMoveTask(t.id, columns[colIdx - 1].id);
                              }}
                              disabled={colIdx === 0}
                              className="text-[10px] text-obsidian-500 hover:text-white disabled:opacity-30 disabled:hover:text-obsidian-500"
                            >
                              ← Move Back
                            </button>
                            <button
                              onClick={() => {
                                if (colIdx < columns.length - 1)
                                  handleMoveTask(t.id, columns[colIdx + 1].id);
                              }}
                              disabled={colIdx === columns.length - 1}
                              className="text-[10px] text-violet-400 hover:text-violet-300 disabled:opacity-30"
                            >
                              Advance →
                            </button>
                          </div>
                        </div>
                      ))
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* TAB 2: STUDY TIMETABLE */}
      {activeTab === 'study' && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Schedule Form */}
          <div className="md:col-span-1">
            <Card variant="surface-2">
              <h3 className="text-sm font-semibold text-white mb-3">Plan Study Block</h3>
              <form onSubmit={handleCreateStudyBlock} className="space-y-3">
                <div>
                  <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                    Subject / Course
                  </label>
                  <input
                    type="text"
                    placeholder="e.g. Computer Science, Skripsi"
                    value={studySubject}
                    onChange={(e) => setStudySubject(e.target.value)}
                    required
                    className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-2.5 py-1.5 text-xs text-white placeholder-obsidian-600 focus:outline-none focus:border-violet-500"
                  />
                </div>

                <div>
                  <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                    Topic / Objective
                  </label>
                  <input
                    type="text"
                    placeholder="e.g. Bab 4 Method Implementation"
                    value={studyTitle}
                    onChange={(e) => setStudyTitle(e.target.value)}
                    required
                    className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-2.5 py-1.5 text-xs text-white placeholder-obsidian-600 focus:outline-none focus:border-violet-500"
                  />
                </div>

                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                      Duration (Mins)
                    </label>
                    <input
                      type="number"
                      value={studyDuration}
                      onChange={(e) => setStudyDuration(e.target.value)}
                      className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-violet-500"
                    />
                  </div>
                  <div>
                    <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                      Priority
                    </label>
                    <select
                      value={studyPriority}
                      onChange={(e) => setStudyPriority(e.target.value)}
                      className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-violet-500"
                    >
                      <option value="low">Low</option>
                      <option value="medium">Medium</option>
                      <option value="high">High</option>
                    </select>
                  </div>
                </div>

                <p className="text-[10px] text-obsidian-500 italic">
                  * Otomatis mengaplikasikan 10 menit padding istirahat untuk mencegah tabrakan jadwal.
                </p>

                <Button type="submit" variant="primary" size="sm" className="w-full mt-2">
                  Schedule Collision-Free Block
                </Button>
              </form>
            </Card>
          </div>

          {/* Schedule List */}
          <div className="md:col-span-2 space-y-3">
            <h2 className="text-sm font-semibold text-white">Upcoming Study Blocks</h2>
            {schedules.length === 0 ? (
              <Card variant="surface-2">
                <div className="py-8 text-center text-xs text-obsidian-500 italic">
                  Belum ada jadwal belajar yang diagendakan.
                </div>
              </Card>
            ) : (
              schedules.map((s) => (
                <div
                  key={s.id}
                  className="bg-obsidian-900 border border-obsidian-800 rounded-lg p-3.5 flex items-center justify-between"
                >
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span className="text-xs font-semibold text-white">{s.title}</span>
                      <Badge variant="cyan" size="sm">
                        {s.subject}
                      </Badge>
                      <Badge variant={priorityColor(s.priority)} size="sm">
                        {s.priority}
                      </Badge>
                    </div>
                    <div className="text-[11px] text-obsidian-400 font-mono">
                      {new Date(s.start_time * 1000).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}{' '}
                      -{' '}
                      {new Date(s.end_time * 1000).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                      {s.rescheduled_from && (
                        <span className="text-amber-400 ml-2 text-[10px]">(Rescheduled)</span>
                      )}
                    </div>
                  </div>

                  <div>
                    {s.is_completed ? (
                      <Badge variant="success" size="sm">
                        Completed
                      </Badge>
                    ) : (
                      <Badge variant="default" size="sm">
                        Upcoming
                      </Badge>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* TAB 3: ACCOUNTABILITY & KETEGASAN */}
      {activeTab === 'accountability' && (
        <div className="space-y-6">
          {/* Mode Selector Cards */}
          <div>
            <h2 className="text-sm font-semibold text-white mb-2">Pilih Mode Ketegasan Hime</h2>
            <p className="text-xs text-obsidian-400 mb-4">
              Filosofi inti Hime: <strong className="text-violet-300">"Hime menagih, bukan mengunci."</strong> Hime tidak pernah mengambil alih kontrol OS, tidak mengunci layar secara paksa, dan selalu menyediakan opsi tunda (snooze) dengan refleksi.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {/* Santai */}
              <div
                onClick={() => handleSetMode('santai')}
                className={`cursor-pointer rounded-lg p-4 border transition-all ${
                  accountability?.mode === 'santai'
                    ? 'bg-violet-950/40 border-violet-500 ring-1 ring-violet-500'
                    : 'bg-obsidian-900 border-obsidian-800 hover:border-obsidian-700'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-white">Mode Santai</span>
                  {accountability?.mode === 'santai' && <Badge variant="success">Active</Badge>}
                </div>
                <p className="text-xs text-obsidian-400 mb-3 leading-relaxed">
                  Pengingat pasif dan ramah. Tidak ada dialog pop-up saat startup, batas waktu tunda (snooze) bebas tanpa batas.
                </p>
                <div className="text-[10px] text-obsidian-500 font-mono">
                  Ideal untuk: Libur, eksplorasi santai, riset mandiri.
                </div>
              </div>

              {/* Biasa */}
              <div
                onClick={() => handleSetMode('biasa')}
                className={`cursor-pointer rounded-lg p-4 border transition-all ${
                  accountability?.mode === 'biasa'
                    ? 'bg-violet-950/40 border-violet-500 ring-1 ring-violet-500'
                    : 'bg-obsidian-900 border-obsidian-800 hover:border-obsidian-700'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-white">Mode Biasa (Default)</span>
                  {accountability?.mode === 'biasa' && <Badge variant="cyan">Active</Badge>}
                </div>
                <p className="text-xs text-obsidian-400 mb-3 leading-relaxed">
                  Pemeriksaan saat startup, ringkasan tugas prioritas, teguran halus jika membuka aplikasi non-kerja terlalu lama.
                </p>
                <div className="text-[10px] text-obsidian-500 font-mono">
                  Ideal untuk: Kerja harian, produktivitas seimbang.
                </div>
              </div>

              {/* Strict */}
              <div
                onClick={() => handleSetMode('strict')}
                className={`cursor-pointer rounded-lg p-4 border transition-all ${
                  accountability?.mode === 'strict'
                    ? 'bg-violet-950/40 border-amber-500 ring-1 ring-amber-500'
                    : 'bg-obsidian-900 border-obsidian-800 hover:border-obsidian-700'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-white">Mode Strict</span>
                  {accountability?.mode === 'strict' && <Badge variant="warning">Active</Badge>}
                </div>
                <p className="text-xs text-obsidian-400 mb-3 leading-relaxed">
                  Rincian tugas mendesak ditampilkan langsung, timer fokus 45 menit aktif, tunda (snooze) memerlukan catatan refleksi singkat.
                </p>
                <div className="text-[10px] text-amber-400/80 font-mono">
                  Ideal untuk: Pekan ujian skripsi, deadline sprint ketat.
                </div>
              </div>
            </div>
          </div>

          {/* Screen Time Privacy & Opt-In */}
          <Card variant="surface-2">
            <h3 className="text-sm font-semibold text-white mb-2">Screen Time Privacy Boundary (SEC Compliance)</h3>
            <div className="flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-white">
                  Opt-in Monitoring Judul Jendela Aktif
                </div>
                <p className="text-[11px] text-obsidian-400 mt-1 max-w-xl">
                  Sesuai kebijakan privasi ketat Hime, judul jendela aplikasi aktif TIDAK PERNAH dicatat tanpa persetujuan eksplisit pengguna. Data ini disimpan lokal 100% pada SQLite mesin pengguna dan tidak dikirim ke cloud.
                </p>
              </div>
              <Button
                variant={accountability?.opt_in_screen_time ? 'danger' : 'secondary'}
                size="sm"
                onClick={handleToggleScreenTimeOptIn}
              >
                {accountability?.opt_in_screen_time ? 'Nonaktifkan Opt-In' : 'Aktifkan Opt-In'}
              </Button>
            </div>
          </Card>

          {/* Check-in Simulator Preview */}
          {checkin && (
            <Card variant="surface-2">
              <h3 className="text-sm font-semibold text-white mb-2">Startup Check-In Preview</h3>
              <div className="bg-obsidian-950/70 border border-obsidian-800 rounded-lg p-4 space-y-3">
                <div className="text-xs font-medium text-violet-300">
                  {checkin.greeting}
                </div>
                <div className="text-[11px] text-obsidian-400">
                  Tugas Mendesak Terdaftar: <strong className="text-white">{checkin.urgent_tasks.length}</strong> | Sesi Belajar Hari Ini: <strong className="text-white">{checkin.today_schedules.length}</strong>
                </div>
                {checkin.snooze_available && (
                  <div className="flex items-center space-x-2 pt-2">
                    <Button variant="secondary" size="sm">
                      Snooze 15m
                    </Button>
                    <Button variant="primary" size="sm">
                      Mulai Sesi Fokus Sekarang
                    </Button>
                  </div>
                )}
              </div>
            </Card>
          )}
        </div>
      )}
    </div>
  );
};
