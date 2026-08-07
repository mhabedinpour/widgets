import { useState, useEffect } from 'react';
import type { SystemStatus, WidgetInfo } from './types';
import { MatrixSimulator } from './components/MatrixSimulator';
import { WidgetEditor } from './components/WidgetEditor';
import {
  Activity,
  Cpu,
  Clock,
  Layers,
  RefreshCw,
  Power,
  Sparkles,
  FileCode,
} from 'lucide-react';
import { WasmModuleManager } from './components/WasmModuleManager';

const DEFAULT_STATUS: SystemStatus = {
  status: 'connecting',
  uptime_ms: 0,
  ip: '0.0.0.0',
  free_heap: 0,
  free_psram: 0,
  widget_count: 0,
};

export function App() {
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [status, setStatus] = useState<SystemStatus>(DEFAULT_STATUS);
  const [widgets, setWidgets] = useState<WidgetInfo[]>([]);
  const [selectedWidgetId, setSelectedWidgetId] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<'manager' | 'wasm'>('manager');
  const [isRefreshing, setIsRefreshing] = useState(false);

  const fetchStatus = async () => {
    try {
      const resStatus = await fetch('/api/status', { signal: AbortSignal.timeout(3000) });
      if (resStatus.ok) {
        const dataStatus = await resStatus.json();
        setStatus(dataStatus);
        setIsConnected(true);
      } else {
        setIsConnected(false);
      }
    } catch {
      setIsConnected(false);
    }
  };

  const fetchWidgets = async () => {
    try {
      const resWidgets = await fetch('/api/widgets', { signal: AbortSignal.timeout(3000) });
      if (resWidgets.ok) {
        const dataWidgets: WidgetInfo[] = await resWidgets.json();
        setWidgets(dataWidgets);
        return dataWidgets;
      }
    } catch (err) {
      console.error('Failed to fetch widgets', err);
    }
    return null;
  };

  const fetchDeviceData = async () => {
    setIsRefreshing(true);
    const results = await Promise.allSettled([fetchStatus(), fetchWidgets()]);
    setIsRefreshing(false);
    
    if (results[1].status === 'fulfilled') {
      return results[1].value;
    }
    return null;
  };

  useEffect(() => {
    const init = async () => {
      const widgetsData = await fetchDeviceData();
      if (widgetsData && widgetsData.length > 0) {
        setSelectedWidgetId(widgetsData[0].id);
      }
    };
    init();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleAddWidget = async (widget: Omit<WidgetInfo, 'id'>) => {
    const res = await fetch('/api/widgets', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: 0, ...widget }),
    });

    if (!res.ok) {
      throw new Error(`Server returned HTTP ${res.status}`);
    }

    const newId = await res.json();
    await fetchDeviceData();
    setSelectedWidgetId(newId);
  };

  const handleReplaceWidget = async (widgetId: number, widget: Omit<WidgetInfo, 'id'>) => {
    const res = await fetch(`/api/widgets/${widgetId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: widgetId, ...widget }),
    });

    if (!res.ok) {
      throw new Error(`Server returned HTTP ${res.status}`);
    }

    await fetchDeviceData();
  };

  const handleRemoveWidget = async (widgetId: number) => {
    const res = await fetch(`/api/widgets/${widgetId}`, {
      method: 'DELETE',
    });

    if (!res.ok) {
      throw new Error(`Server returned HTTP ${res.status}`);
    }

    const widgetsData = await fetchDeviceData();
    if (widgetsData && widgetsData.length > 0) {
      setSelectedWidgetId(widgetsData[0].id);
    } else {
      setSelectedWidgetId(null);
    }
  };

  const handleUploadWasm = async (file: File) => {
    const res = await fetch(`/api/upload/${file.name}`, {
      method: 'POST',
      body: file,
    });

    if (!res.ok) {
      throw new Error(`Server returned HTTP ${res.status}`);
    }

    await fetchDeviceData();
  };

  const handleReboot = async () => {
    if (confirm('Are you sure you want to reboot the ESP32-S3 LED Matrix hardware?')) {
      await fetch('/api/reboot', { method: 'POST' });
      alert('Reboot signal sent!');
    }
  };

  const formatUptime = (ms: number) => {
    const sec = Math.floor(ms / 1000);
    const m = Math.floor(sec / 60);
    const h = Math.floor(m / 60);
    return `${h}h ${m % 60}m ${sec % 60}s`;
  };

  return (
    <div>
      {/* Top Navbar */}
      <nav className="navbar">
        <div className="brand">
          <div className="brand-icon">
            <Sparkles size={20} color="#000" />
          </div>
          <div>
            <div className="brand-title">LED Matrix OS</div>
            <div className="brand-subtitle">WASM Admin Dashboard</div>
          </div>
        </div>

        <div className="connection-bar">
          <div className="status-badge">
            <span className={`status-dot ${isConnected ? 'online' : 'offline'}`} />
            {isConnected ? 'Connected (ESP32-S3)' : 'Device Offline'}
          </div>
        </div>

        <div style={{ display: 'flex', gap: '0.75rem' }}>
          <button className="btn btn-secondary" onClick={fetchDeviceData} disabled={isRefreshing}>
            <RefreshCw className={isRefreshing ? 'spin' : ''} size={16} />
            Refresh
          </button>
          <button className="btn btn-danger" onClick={handleReboot}>
            <Power size={16} />
            Reboot Device
          </button>
        </div>
      </nav>

      {/* Main Container */}
      <div className="container">
        {/* Quick Metrics */}
        <div className="metrics-grid">
          <div className="glass-panel metric-card">
            <div className="metric-header">
              <span>System Uptime</span>
              <Clock size={16} color="var(--accent-cyan)" />
            </div>
            <div className="metric-value">{formatUptime(status.uptime_ms)}</div>
            <div className="metric-subtext">SNTP Synced</div>
          </div>

          <div className="glass-panel metric-card">
            <div className="metric-header">
              <span>ESP32-S3 Memory</span>
              <Cpu size={16} color="var(--accent-purple)" />
            </div>
            <div className="metric-value">
              {status.free_psram > 0 ? `${(status.free_psram / (1024 * 1024)).toFixed(1)} MB PSRAM` : 'Internal RAM'}
            </div>
            <div className="metric-subtext">Free Heap: {(status.free_heap / 1024).toFixed(1)} KB</div>
          </div>

          <div className="glass-panel metric-card">
            <div className="metric-header">
              <span>Active WASM Widgets</span>
              <Layers size={16} color="var(--accent-amber)" />
            </div>
            <div className="metric-value">{widgets.length} Loaded</div>
            <div className="metric-subtext">Sandbox: Wasmi no_std</div>
          </div>
        </div>

        {/* Workspace Layout */}
        <div className="workspace-grid">
          {/* Left Column: Live Matrix Preview */}
          <div className="matrix-section">
            <MatrixSimulator
              widgets={widgets}
              selectedWidgetId={selectedWidgetId}
              onSelectWidget={(id) => setSelectedWidgetId(id)}
            />
          </div>

          {/* Right Column: Tab View (Widget Editor / WASM Manager) */}
          <div className="glass-panel" style={{ padding: '1.5rem' }}>
            <div className="tabs-header">
              <button
                className={`tab-btn ${activeTab === 'manager' ? 'active' : ''}`}
                onClick={() => setActiveTab('manager')}
              >
                <Activity size={16} style={{ display: 'inline', marginRight: '0.4rem', verticalAlign: 'middle' }} />
                Widget Manager
              </button>
              <button
                className={`tab-btn ${activeTab === 'wasm' ? 'active' : ''}`}
                onClick={() => setActiveTab('wasm')}
              >
                <FileCode size={16} style={{ display: 'inline', marginRight: '0.4rem', verticalAlign: 'middle' }} />
                WASM Modules
              </button>
            </div>

            {activeTab === 'manager' ? (
              <WidgetEditor
                widgets={widgets}
                selectedWidgetId={selectedWidgetId}
                onSelectWidget={(id) => setSelectedWidgetId(id)}
                onAddWidget={handleAddWidget}
                onReplaceWidget={handleReplaceWidget}
                onRemoveWidget={handleRemoveWidget}
              />
            ) : (
              <WasmModuleManager onUpload={handleUploadWasm} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
