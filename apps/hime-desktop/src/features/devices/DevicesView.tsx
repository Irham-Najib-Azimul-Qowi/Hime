import React, { useState, useEffect, useCallback } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  getDevicesList,
  startDevicePairing,
  revokeDevice,
  getDeviceStorage,
  getDeviceTransfers,
} from '../../services/ipcService';
import {
  PairedDevice,
  PairingSession,
  DeviceStorageSummary,
  FileTransfer,
} from '../../types/devices';

export const DevicesView: React.FC = () => {
  const [devices, setDevices] = useState<PairedDevice[]>([]);
  const [activeSession, setActiveSession] = useState<PairingSession | null>(null);
  const [selectedDeviceStorage, setSelectedDeviceStorage] = useState<DeviceStorageSummary | null>(null);
  const [transfers, setTransfers] = useState<FileTransfer[]>([]);
  const [loading, setLoading] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [devList, xferList] = await Promise.all([
        getDevicesList(),
        getDeviceTransfers(),
      ]);
      setDevices(devList);
      setTransfers(xferList);

      const activeDev = devList.find((d: PairedDevice) => d.status === 'active');
      if (activeDev) {
        const storageSummary = await getDeviceStorage(activeDev.id);
        setSelectedDeviceStorage(storageSummary);
      }
    } catch (err: any) {
      setStatusMessage(`Error loading devices: ${err?.message || err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleStartPairing = async () => {
    try {
      const session = await startDevicePairing(300);
      setActiveSession(session);
      setStatusMessage('Pairing session generated. Scan QR or enter code on your Android device.');
    } catch (err: any) {
      setStatusMessage(`Failed to initiate pairing: ${err?.message || err}`);
    }
  };

  const handleRevoke = async (deviceId: string) => {
    if (!window.confirm(`Are you sure you want to revoke device access for ${deviceId}? This action takes effect immediately.`)) {
      return;
    }
    try {
      await revokeDevice(deviceId);
      setStatusMessage(`Device ${deviceId} access revoked successfully.`);
      await loadData();
    } catch (err: any) {
      setStatusMessage(`Revocation failed: ${err?.message || err}`);
    }
  };

  return (
    <div className="flex flex-col h-full overflow-y-auto p-6 space-y-6 text-obsidian-100">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-obsidian-800 pb-4">
        <div>
          <div className="flex items-center space-x-2">
            <span className="text-xs uppercase tracking-widest text-violet-400 font-mono">Mobile & Devices (UR-02 / UR-04)</span>
          </div>
          <h1 className="text-xl font-semibold text-white mt-1">Paired Devices & Android Companion</h1>
          <p className="text-xs text-obsidian-400 mt-0.5">
            Mutual TLS encrypted device link, remote approval management, and storage intelligence.
          </p>
        </div>

        <div className="flex items-center space-x-3">
          <Button
            variant="secondary"
            size="sm"
            onClick={loadData}
            disabled={loading}
          >
            {loading ? 'Refreshing...' : 'Refresh'}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={handleStartPairing}
          >
            + Pair New Android Device
          </Button>
        </div>
      </div>

      {statusMessage && (
        <div className="p-3 bg-obsidian-900 border border-obsidian-700 rounded-lg text-xs text-obsidian-300 flex justify-between items-center">
          <span>{statusMessage}</span>
          <button onClick={() => setStatusMessage(null)} className="text-obsidian-500 hover:text-white ml-2">✕</button>
        </div>
      )}

      {/* Active Pairing Modal / Banner */}
      {activeSession && (
        <Card className="p-6 bg-obsidian-900/90 border-violet-500/50 relative overflow-hidden">
          <div className="flex items-start justify-between">
            <div>
              <span className="text-xs font-mono uppercase text-violet-400 font-bold">Secure Single-Use Pairing Session</span>
              <h2 className="text-lg font-semibold text-white mt-1">Scan or Enter Confirmation Code</h2>
              <p className="text-xs text-obsidian-400 mt-1 max-w-xl">
                Open Hime Mobile Companion on your Android phone and scan this session or type the 6-digit confirmation code.
              </p>
            </div>
            <Button variant="ghost" size="sm" onClick={() => setActiveSession(null)}>Close</Button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6 items-center">
            {/* Visual QR representation placeholder */}
            <div className="flex flex-col items-center justify-center p-6 bg-obsidian-950 rounded-xl border border-obsidian-800">
              <div className="w-36 h-36 bg-obsidian-900 border-2 border-dashed border-violet-400/60 rounded-lg flex items-center justify-center p-3 text-center">
                <span className="font-mono text-xs text-violet-300 break-all select-all">
                  QR Payload<br/>
                  {activeSession.server_address}:{activeSession.server_port}<br/>
                  CA: {activeSession.ca_fingerprint.slice(0, 16)}...
                </span>
              </div>
              <span className="text-[11px] text-obsidian-400 mt-3 font-mono">
                Expires at: {new Date(activeSession.expires_at * 1000).toLocaleTimeString()}
              </span>
            </div>

            {/* Confirmation Code Card */}
            <div className="p-6 bg-obsidian-950 rounded-xl border border-obsidian-800 space-y-4">
              <div>
                <span className="text-xs text-obsidian-400">Two-Way Confirmation Code:</span>
                <div className="text-3xl font-mono font-bold tracking-widest text-emerald-400 mt-1">
                  {activeSession.confirmation_code}
                </div>
              </div>

              <div>
                <span className="text-xs text-obsidian-400">One-Time Pairing Token:</span>
                <div className="text-xs font-mono text-obsidian-300 break-all bg-obsidian-900 p-2 rounded mt-1 select-all">
                  {activeSession.pairing_token}
                </div>
              </div>

              <div className="text-[11px] text-obsidian-500">
                Anti-Replay: Token is single-use and invalidates immediately upon consumption.
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Paired Devices List */}
      <div>
        <h2 className="text-sm font-semibold text-white uppercase tracking-wider mb-3">Paired Devices ({devices.length})</h2>

        {devices.length === 0 ? (
          <div className="p-8 text-center bg-obsidian-900/50 rounded-xl border border-obsidian-800">
            <p className="text-sm text-obsidian-400">No mobile devices currently paired.</p>
            <p className="text-xs text-obsidian-500 mt-1">Click "+ Pair New Android Device" above to connect your phone.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {devices.map((dev) => (
              <Card key={dev.id} className="p-5 bg-obsidian-900/70 border-obsidian-800">
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center space-x-2">
                      <span className="font-semibold text-white">{dev.device_name}</span>
                      <Badge variant={dev.status === 'active' ? 'success' : 'danger'}>
                        {dev.status.toUpperCase()}
                      </Badge>
                    </div>
                    <div className="text-xs text-obsidian-400 mt-1">
                      Type: <span className="text-obsidian-300 capitalize">{dev.device_type}</span> • ID: <span className="font-mono text-obsidian-400">{dev.id}</span>
                    </div>
                  </div>

                  {dev.status === 'active' && (
                    <Button
                      variant="danger"
                      size="sm"
                      onClick={() => handleRevoke(dev.id)}
                    >
                      Revoke
                    </Button>
                  )}
                </div>

                <div className="mt-4 pt-3 border-t border-obsidian-800/80 space-y-1 text-xs">
                  <div className="flex justify-between">
                    <span className="text-obsidian-400">Client Certificate:</span>
                    <span className="font-mono text-obsidian-300 truncate max-w-[200px]" title={dev.cert_fingerprint}>
                      {dev.cert_fingerprint}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-obsidian-400">Paired:</span>
                    <span className="text-obsidian-300">{new Date(dev.paired_at * 1000).toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-obsidian-400">Last Seen:</span>
                    <span className="text-obsidian-300">{new Date(dev.last_seen_at * 1000).toLocaleString()}</span>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>

      {/* Selected Phone Storage Summary */}
      {selectedDeviceStorage && (
        <Card className="p-5 bg-obsidian-900/70 border-obsidian-800 space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-white uppercase tracking-wider">
              Mobile Storage Overview ({selectedDeviceStorage.device_id})
            </h2>
            <span className="text-xs text-obsidian-400 font-mono">
              Updated: {new Date(selectedDeviceStorage.updated_at * 1000).toLocaleTimeString()}
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2">
            <div className="p-3 bg-obsidian-950 rounded-lg border border-obsidian-800">
              <span className="text-xs text-obsidian-400">Total Storage</span>
              <div className="text-base font-bold text-white mt-1">
                {(selectedDeviceStorage.total_bytes / 1_000_000_000).toFixed(1)} GB
              </div>
            </div>

            <div className="p-3 bg-obsidian-950 rounded-lg border border-obsidian-800">
              <span className="text-xs text-obsidian-400">Free Space</span>
              <div className="text-base font-bold text-emerald-400 mt-1">
                {(selectedDeviceStorage.free_bytes / 1_000_000_000).toFixed(1)} GB
              </div>
            </div>

            <div className="p-3 bg-obsidian-950 rounded-lg border border-obsidian-800">
              <span className="text-xs text-obsidian-400">Used Ratio</span>
              <div className="text-base font-bold text-violet-400 mt-1">
                {(((selectedDeviceStorage.total_bytes - selectedDeviceStorage.free_bytes) / selectedDeviceStorage.total_bytes) * 100).toFixed(0)}%
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Cross-Device Transfers Journal */}
      <div>
        <h2 className="text-sm font-semibold text-white uppercase tracking-wider mb-3">
          File Transfers ({transfers.length})
        </h2>

        {transfers.length === 0 ? (
          <div className="p-6 text-center bg-obsidian-900/40 rounded-xl border border-obsidian-800">
            <p className="text-xs text-obsidian-400">No transfers recorded yet.</p>
          </div>
        ) : (
          <div className="overflow-x-auto rounded-xl border border-obsidian-800 bg-obsidian-900/60">
            <table className="w-full text-xs text-left text-obsidian-300">
              <thead className="bg-obsidian-950 text-obsidian-400 font-mono uppercase text-[10px] border-b border-obsidian-800">
                <tr>
                  <th className="px-4 py-3">File Name</th>
                  <th className="px-4 py-3">Direction</th>
                  <th className="px-4 py-3">Size</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="px-4 py-3">SHA-256</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-obsidian-800/60">
                {transfers.map((xfer) => (
                  <tr key={xfer.id} className="hover:bg-obsidian-800/30">
                    <td className="px-4 py-3 font-medium text-white">{xfer.file_name}</td>
                    <td className="px-4 py-3 uppercase font-mono text-[10px]">{xfer.direction}</td>
                    <td className="px-4 py-3">{(xfer.file_size / 1_000_000).toFixed(2)} MB</td>
                    <td className="px-4 py-3">
                      <Badge variant={xfer.status === 'completed' ? 'success' : 'warning'}>
                        {xfer.status}
                      </Badge>
                    </td>
                    <td className="px-4 py-3 font-mono text-[10px] text-obsidian-400 truncate max-w-[150px]">
                      {xfer.sha256}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
