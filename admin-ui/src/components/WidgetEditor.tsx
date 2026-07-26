import React, { useState } from 'react';
import type { WidgetInfo } from '../types';
import { Settings, Save, Sliders, RefreshCw } from 'lucide-react';

interface WidgetEditorProps {
  widgets: WidgetInfo[];
  selectedWidgetId: number | null;
  onSelectWidget: (id: number) => void;
  onUpdateConfig: (widgetId: number, config: Record<string, string>) => Promise<void>;
}

export const WidgetEditor: React.FC<WidgetEditorProps> = ({
  widgets,
  selectedWidgetId,
  onSelectWidget,
  onUpdateConfig,
}) => {
  const selectedWidget = widgets.find((w) => w.id === selectedWidgetId) || widgets[0];

  const [editingConfig, setEditingConfig] = useState<Record<string, string>>(
    selectedWidget ? { ...selectedWidget.config } : {}
  );
  const [isSaving, setIsSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  React.useEffect(() => {
    if (selectedWidget) {
      setEditingConfig({ ...selectedWidget.config });
    }
  }, [selectedWidgetId]);

  const handleConfigChange = (key: string, value: string) => {
    setEditingConfig((prev) => ({ ...prev, [key]: value }));
  };

  const handleSave = async () => {
    if (!selectedWidget) return;
    setIsSaving(true);
    setMessage(null);
    try {
      await onUpdateConfig(selectedWidget.id, editingConfig);
      setMessage('Configuration saved successfully!');
    } catch (err: any) {
      setMessage(`Save failed: ${err.message}`);
    } finally {
      setIsSaving(false);
    }
  };

  if (!selectedWidget) {
    return <div className="glass-panel" style={{ padding: '2rem' }}>No widgets configured.</div>;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <div className="glass-panel widget-card">
        <div className="widget-card-header">
          <div className="widget-title">
            <Settings size={20} color="var(--accent-cyan)" />
            Widget #{selectedWidget.id}: {selectedWidget.type}
            <span className="widget-type-badge">{selectedWidget.type}.ts</span>
          </div>

          <div style={{ display: 'flex', gap: '0.5rem' }}>
            {widgets.map((w) => (
              <button
                key={w.id}
                className={`btn ${w.id === selectedWidget.id ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => onSelectWidget(w.id)}
                style={{ padding: '0.3rem 0.6rem', fontSize: '0.75rem' }}
              >
                #{w.id} {w.type}
              </button>
            ))}
          </div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <span className="form-label" style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
            <Sliders size={14} color="var(--accent-purple)" /> Viewport Coordinates (64×64 Panel Bounds)
          </span>
          <div className="coords-grid">
            <div className="form-group">
              <label className="form-label">X Offset</label>
              <input className="form-input" type="number" value={selectedWidget.x} readOnly />
            </div>
            <div className="form-group">
              <label className="form-label">Y Offset</label>
              <input className="form-input" type="number" value={selectedWidget.y} readOnly />
            </div>
            <div className="form-group">
              <label className="form-label">Width</label>
              <input className="form-input" type="number" value={selectedWidget.width} readOnly />
            </div>
            <div className="form-group">
              <label className="form-label">Height</label>
              <input className="form-input" type="number" value={selectedWidget.height} readOnly />
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '0.5rem' }}>
          <span className="form-label">Dynamic Key-Value Configuration</span>
          {Object.keys(editingConfig).length === 0 ? (
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
              No custom config parameters defined for this widget.
            </div>
          ) : (
            <table className="config-kv-table">
              <thead>
                <tr>
                  <th>Parameter Key</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                {Object.entries(editingConfig).map(([key, val]) => (
                  <tr key={key}>
                    <td style={{ fontFamily: 'var(--font-mono)', color: 'var(--accent-cyan)' }}>{key}</td>
                    <td>
                      <input
                        className="form-input"
                        style={{ width: '100%' }}
                        value={val}
                        onChange={(e) => handleConfigChange(key, e.target.value)}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {message && (
          <div style={{
            fontSize: '0.85rem',
            padding: '0.5rem 0.75rem',
            borderRadius: '6px',
            background: message.includes('failed') ? 'rgba(255,71,87,0.15)' : 'rgba(0,255,136,0.15)',
            color: message.includes('failed') ? '#ff4757' : 'var(--accent-emerald)',
            border: `1px solid ${message.includes('failed') ? '#ff4757' : 'var(--accent-emerald)'}`
          }}>
            {message}
          </div>
        )}

        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '0.5rem' }}>
          <button className="btn btn-primary" onClick={handleSave} disabled={isSaving}>
            {isSaving ? <RefreshCw className="spin" size={16} /> : <Save size={16} />}
            Apply & Save to Firmware
          </button>
        </div>
      </div>
    </div>
  );
};
