import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  createDocumentReport,
  mergePdfs,
  listDocumentJobs,
  discoverArtcraft,
} from '../../services/ipcService';
import { DocumentJob, ArtcraftDiscoveryStatus } from '../../types/documents';

export const DocumentsView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'generator' | 'pdf' | 'artcraft' | 'jobs'>('generator');
  const [jobs, setJobs] = useState<DocumentJob[]>([]);
  const [artcraftStatus, setArtcraftStatus] = useState<ArtcraftDiscoveryStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // Report Generator Form State
  const [templateId, setTemplateId] = useState('academic-id');
  const [docTitle, setDocTitle] = useState('Implementasi Algoritma Pengurutan Adaptif');
  const [docAuthor, setDocAuthor] = useState('Mahasiswa Teknik Informatika');
  const [docInstitution, setDocInstitution] = useState('Fakultas Ilmu Komputer, Universitas Hime');
  const [docAbstract, setDocAbstract] = useState(
    'Penelitian ini menyajikan evaluasi komprehensif terhadap arsitektur agen AI mandiri dan antarmuka desktop modern.'
  );
  const [section1Heading, setSection1Heading] = useState('Pendahuluan');
  const [section1Body, setSection1Body] = useState(
    'Perkembangan sistem operasi personal menuntut integrasi agen AI yang tidak invasif dan menjamin kedaulatan data pengguna.'
  );
  const [targetFormat, setTargetFormat] = useState('docx');
  const [isGenerating, setIsGenerating] = useState(false);

  // PDF Merge Form State
  const [pdfPath1, setPdfPath1] = useState('');
  const [pdfPath2, setPdfPath2] = useState('');
  const [pdfOutputPath, setPdfOutputPath] = useState('');
  const [isMerging, setIsMerging] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [jobList, acStatus] = await Promise.all([
        listDocumentJobs(20),
        discoverArtcraft(),
      ]);
      setJobs(jobList);
      setArtcraftStatus(acStatus);
    } catch (err: any) {
      setStatusMessage(`Error loading document data: ${err?.message || err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleGenerateReport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!docTitle.trim() || !docAuthor.trim()) return;

    setIsGenerating(true);
    try {
      const sections = [
        {
          heading: section1Heading,
          content: section1Body,
        },
      ];

      const res = await createDocumentReport(
        templateId,
        docTitle,
        docAuthor,
        docInstitution,
        docAbstract,
        sections,
        targetFormat
      );
      setStatusMessage(`Dokumen berhasil dibuat: ${res.output_path}`);
      await loadData();
      setActiveTab('jobs');
    } catch (err: any) {
      setStatusMessage(`Gagal membuat dokumen: ${err?.message || err}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleMergePdfs = async (e: React.FormEvent) => {
    e.preventDefault();
    const paths = [pdfPath1.trim(), pdfPath2.trim()].filter(Boolean);
    if (paths.length < 2) {
      setStatusMessage('Minimal 2 berkas PDF diperlukan untuk penggabungan.');
      return;
    }

    setIsMerging(true);
    try {
      const res = await mergePdfs(paths, pdfOutputPath.trim() || undefined);
      setStatusMessage(`PDF berhasil digabungkan: ${res.output_path}`);
      setPdfPath1('');
      setPdfPath2('');
      setPdfOutputPath('');
      await loadData();
      setActiveTab('jobs');
    } catch (err: any) {
      setStatusMessage(`Gagal menggabungkan PDF: ${err?.message || err}`);
    } finally {
      setIsMerging(false);
    }
  };

  return (
    <div className="flex flex-col h-full overflow-y-auto p-6 space-y-6 text-obsidian-100">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-obsidian-800 pb-4">
        <div>
          <div className="flex items-center space-x-2">
            <span className="text-xs uppercase tracking-widest text-violet-400 font-mono">
              Document Suite & Artcraft Interop (UR-06 / UR-14 / B3 / B4)
            </span>
          </div>
          <h1 className="text-xl font-semibold text-white mt-1">Document Intelligence & Suite</h1>
          <p className="text-xs text-obsidian-400 mt-0.5">
            Academic Indonesian standard templates (TNR 12, margin 4-3-3-3), PDF processing, and Artcraft format exchange.
          </p>
        </div>

        <div className="flex items-center space-x-3">
          <Button variant="secondary" size="sm" onClick={loadData} disabled={loading}>
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

      {/* Sub Tabs */}
      <div className="flex space-x-2 border-b border-obsidian-800 pb-2">
        <button
          onClick={() => setActiveTab('generator')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'generator'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          Document Generator
        </button>
        <button
          onClick={() => setActiveTab('pdf')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'pdf'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          PDF Tools
        </button>
        <button
          onClick={() => setActiveTab('artcraft')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'artcraft'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          Artcraft Interoperability
        </button>
        <button
          onClick={() => setActiveTab('jobs')}
          className={`px-4 py-1.5 text-xs font-medium rounded-md transition-colors ${
            activeTab === 'jobs'
              ? 'bg-violet-600 text-white'
              : 'text-obsidian-400 hover:text-white hover:bg-obsidian-800'
          }`}
        >
          Job History ({jobs.length})
        </button>
      </div>

      {/* TAB 1: GENERATOR */}
      {activeTab === 'generator' && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-2 space-y-4">
            <Card variant="surface-2">
              <h3 className="text-sm font-semibold text-white mb-3">Create Structured Document</h3>
              <form onSubmit={handleGenerateReport} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                      Template
                    </label>
                    <select
                      value={templateId}
                      onChange={(e) => setTemplateId(e.target.value)}
                      className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500"
                    >
                      <option value="academic-id">
                        Makalah Akademik Indonesia (TNR 12, 1.5, Margin 4-3-3-3)
                      </option>
                      <option value="modern-report">
                        Modern Executive Report (Calibri 11, Margin 2.5cm)
                      </option>
                    </select>
                  </div>
                  <div>
                    <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                      Target Format
                    </label>
                    <select
                      value={targetFormat}
                      onChange={(e) => setTargetFormat(e.target.value)}
                      className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500"
                    >
                      <option value="docx">Microsoft Word (.docx)</option>
                      <option value="html">HTML5 Semantic Report (.html)</option>
                    </select>
                  </div>
                </div>

                <div>
                  <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                    Document Title
                  </label>
                  <input
                    type="text"
                    value={docTitle}
                    onChange={(e) => setDocTitle(e.target.value)}
                    required
                    className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500"
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                      Author Name
                    </label>
                    <input
                      type="text"
                      value={docAuthor}
                      onChange={(e) => setDocAuthor(e.target.value)}
                      required
                      className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500"
                    />
                  </div>
                  <div>
                    <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                      Institution / Affiliation
                    </label>
                    <input
                      type="text"
                      value={docInstitution}
                      onChange={(e) => setDocInstitution(e.target.value)}
                      className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500"
                    />
                  </div>
                </div>

                <div>
                  <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                    Abstract (Ringkasan)
                  </label>
                  <textarea
                    rows={3}
                    value={docAbstract}
                    onChange={(e) => setDocAbstract(e.target.value)}
                    className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500 font-sans"
                  />
                </div>

                <div className="space-y-2 border-t border-obsidian-800 pt-3">
                  <span className="text-xs font-semibold text-white block">Main Section</span>
                  <input
                    type="text"
                    placeholder="Section Heading"
                    value={section1Heading}
                    onChange={(e) => setSection1Heading(e.target.value)}
                    className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-violet-500"
                  />
                  <textarea
                    rows={4}
                    placeholder="Section Content"
                    value={section1Body}
                    onChange={(e) => setSection1Body(e.target.value)}
                    className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white focus:outline-none focus:border-violet-500 font-sans"
                  />
                </div>

                <Button
                  type="submit"
                  variant="primary"
                  size="md"
                  disabled={isGenerating}
                  className="w-full"
                >
                  {isGenerating ? 'Generating & Verifying...' : 'Generate Compliant Document'}
                </Button>
              </form>
            </Card>
          </div>

          {/* Template Specs Inspector */}
          <div className="md:col-span-1 space-y-4">
            <Card variant="surface-2">
              <h3 className="text-sm font-semibold text-white mb-2">Academic ID Specifications</h3>
              <div className="text-xs space-y-2 text-obsidian-300">
                <p>
                  Sesuai standar penulisan skripsi & tugas akhir perguruan tinggi di Indonesia:
                </p>
                <ul className="list-disc list-inside space-y-1 text-[11px] text-obsidian-400">
                  <li>Font: <span className="text-white">Times New Roman 12 pt</span></li>
                  <li>Spasi: <span className="text-white">1.5 line spacing</span></li>
                  <li>Margin Kiri: <span className="text-white">4 cm (penjilidan)</span></li>
                  <li>Margin Atas/Kanan/Bawah: <span className="text-white">3 cm</span></li>
                  <li>Daftar Isi Otomatis (&gt; 5 halaman)</li>
                  <li>Halaman Sampul Standar</li>
                </ul>
                <div className="pt-2 border-t border-obsidian-800 text-[10px] text-emerald-400 font-mono">
                  ✓ Verified against UR-14 requirements
                </div>
              </div>
            </Card>
          </div>
        </div>
      )}

      {/* TAB 2: PDF TOOLS */}
      {activeTab === 'pdf' && (
        <div className="max-w-2xl space-y-4">
          <Card variant="surface-2">
            <h3 className="text-sm font-semibold text-white mb-3">Merge PDF Documents</h3>
            <form onSubmit={handleMergePdfs} className="space-y-3">
              <div>
                <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                  PDF File 1 Absolute Path
                </label>
                <input
                  type="text"
                  placeholder="C:\Users\Satoshi\Documents\chapter1.pdf"
                  value={pdfPath1}
                  onChange={(e) => setPdfPath1(e.target.value)}
                  className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-violet-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                  PDF File 2 Absolute Path
                </label>
                <input
                  type="text"
                  placeholder="C:\Users\Satoshi\Documents\chapter2.pdf"
                  value={pdfPath2}
                  onChange={(e) => setPdfPath2(e.target.value)}
                  className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-violet-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-obsidian-400 uppercase tracking-wider block mb-1">
                  Target Merged Output Path (Optional)
                </label>
                <input
                  type="text"
                  placeholder="C:\Users\Satoshi\Documents\merged_output.pdf"
                  value={pdfOutputPath}
                  onChange={(e) => setPdfOutputPath(e.target.value)}
                  className="w-full bg-obsidian-950 border border-obsidian-800 rounded px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-violet-500"
                />
              </div>

              <Button
                type="submit"
                variant="primary"
                size="sm"
                disabled={isMerging}
                className="w-full mt-2"
              >
                {isMerging ? 'Merging PDF Files...' : 'Merge PDF Documents'}
              </Button>
            </form>
          </Card>
        </div>
      )}

      {/* TAB 3: ARTCRAFT INTEROP */}
      {activeTab === 'artcraft' && (
        <div className="space-y-4">
          <Card variant="surface-2">
            <h3 className="text-sm font-semibold text-white mb-2">Artcraft Parity & Environment Discovery (UR-06 / B4)</h3>
            <div className="space-y-4 text-xs">
              <p className="text-obsidian-300 leading-relaxed">
                Hime menyediakan adapter interoperabilitas dengan ekosistem Artcraft (WordCraft / FigCraft). Jika aplikasi Artcraft terpasang secara lokal, Hime dapat bertukar dokumen melalui format terbuka tanpa memerlukan otomasi antarmuka GUI yang rapuh.
              </p>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="bg-obsidian-950 border border-obsidian-800 rounded-lg p-4 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-white">WordCraft Suite</span>
                    <Badge variant={artcraftStatus?.wordcraft_available ? 'success' : 'default'}>
                      {artcraftStatus?.wordcraft_available ? 'Installed' : 'Not Detected'}
                    </Badge>
                  </div>
                  <p className="text-[11px] text-obsidian-400">
                    {artcraftStatus?.wordcraft_path || 'Aplikasi WordCraft tidak ditemukan di registri lokal.'}
                  </p>
                </div>

                <div className="bg-obsidian-950 border border-obsidian-800 rounded-lg p-4 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-white">FigCraft Design</span>
                    <Badge variant={artcraftStatus?.figcraft_available ? 'success' : 'default'}>
                      {artcraftStatus?.figcraft_available ? 'Installed' : 'Not Detected'}
                    </Badge>
                  </div>
                  <p className="text-[11px] text-obsidian-400">
                    {artcraftStatus?.figcraft_path || 'Aplikasi FigCraft tidak ditemukan di registri lokal.'}
                  </p>
                </div>
              </div>

              <div className="pt-3 border-t border-obsidian-800">
                <span className="text-[11px] uppercase tracking-wider text-obsidian-400 block mb-2">
                  Supported Open Interchange Formats
                </span>
                <div className="flex gap-2">
                  {artcraftStatus?.supported_interchanges.map((fmt) => (
                    <Badge key={fmt} variant="cyan" size="sm">
                      .{fmt.toUpperCase()}
                    </Badge>
                  ))}
                </div>
              </div>
            </div>
          </Card>
        </div>
      )}

      {/* TAB 4: JOB HISTORY */}
      {activeTab === 'jobs' && (
        <div className="space-y-3">
          <h2 className="text-sm font-semibold text-white">Document Generation & Processing Jobs</h2>
          {jobs.length === 0 ? (
            <Card variant="surface-2">
              <div className="py-8 text-center text-xs text-obsidian-500 italic">
                Belum ada pekerjaan pemrosesan dokumen yang tercatat.
              </div>
            </Card>
          ) : (
            jobs.map((job) => (
              <div
                key={job.id}
                className="bg-obsidian-900 border border-obsidian-800 rounded-lg p-3.5 flex items-center justify-between"
              >
                <div className="space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="text-xs font-semibold text-white">{job.id}</span>
                    <Badge variant="cyan" size="sm">
                      {job.template_id}
                    </Badge>
                    <Badge variant="default" size="sm">
                      {job.target_format.toUpperCase()}
                    </Badge>
                    <Badge
                      variant={
                        job.status === 'completed'
                          ? 'success'
                          : job.status === 'failed'
                          ? 'danger'
                          : 'warning'
                      }
                      size="sm"
                    >
                      {job.status}
                    </Badge>
                  </div>
                  <div className="text-[11px] text-obsidian-400 font-mono">
                    Output: {job.output_path}
                  </div>
                </div>

                <div className="text-[11px] text-obsidian-500 font-mono">
                  {new Date(job.created_at * 1000).toLocaleString()}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
};
