import React, { useState, useEffect } from 'react';
import type { WidgetInfo } from '../types';
import { Settings, Save, Sliders, RefreshCw, Trash2, Plus, X } from 'lucide-react';

interface WidgetEditorProps {
  widgets: WidgetInfo[];
  selectedWidgetId: number | null;
  onSelectWidget: (id: number | null) => void;
  onAddWidget: (widget: Omit<WidgetInfo, 'id'>) => Promise<void>;
  onReplaceWidget: (widgetId: number, widget: Omit<WidgetInfo, 'id'>) => Promise<void>;
  onRemoveWidget: (widgetId: number) => Promise<void>;
}

export const WidgetEditor: React.FC<WidgetEditorProps> = ({
  widgets,
  selectedWidgetId,
  onSelectWidget,
  onAddWidget,
  onReplaceWidget,
  onRemoveWidget,
}) => {
  const isNew = selectedWidgetId === null;
  const selectedWidget = widgets.find((w) => w.id === selectedWidgetId);

  const [type, setType] = useState('');
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [width, setWidth] = useState(32);
  const [height, setHeight] = useState(32);
  const [editingConfig, setEditingConfig] = useState<Record<string, string>>({});
  const [newConfigKey, setNewConfigKey] = useState('');
  const [newConfigValue, setNewConfigValue] = useState('');

  const [isSaving, setIsSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    if (selectedWidget) {
      setType(selectedWidget.type);
      setX(selectedWidget.x);
      setY(selectedWidget.y);
      setWidth(selectedWidget.width);
      setHeight(selectedWidget.height);
      setEditingConfig({ ...selectedWidget.config });
    } else {
      setType('');
      setX(0);
      setY(0);
      setWidth(32);
      setHeight(32);
      setEditingConfig({});
    }
    setMessage(null);
  }, [selectedWidgetId, selectedWidget]);

  const handleConfigChange = (key: string, value: string) => {
    setEditingConfig((prev) => ({ ...prev, [key]: value }));
  };

  const handleAddConfigPair = () => {
    if (newConfigKey && !editingConfig[newConfigKey]) {
      setEditingConfig((prev) => ({ ...prev, [newConfigKey]: newConfigValue }));
      setNewConfigKey('');
      setNewConfigValue('');
    }
  };

  const handleRemoveConfigPair = (key: string) => {
    const next = { ...editingConfig };
    delete next[key];
    setEditingConfig(next);
  };

  const handleSave = async () => {
    setIsSaving(true);
    setMessage(null);
    try {
      const widgetData = {
        type,
        x,
        y,
        width,
        height,
        config: editingConfig,
      };

      if (isNew) {
        await onAddWidget(widgetData);
        setMessage('Widget created successfully!');
      } else {
        await onReplaceWidget(selectedWidgetId!, widgetData);
        setMessage('Widget replaced successfully!');
      }
    } catch (err: any) {
      setMessage(`Operation failed: ${err.message}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedWidgetId) return;
    if (!confirm('Are you sure you want to remove this widget?')) return;

    setIsSaving(true);
    setMessage(null);
    try {
      await onRemoveWidget(selectedWidgetId);
      setMessage('Widget removed successfully!');
    } catch (err: any) {
      setMessage(`Removal failed: ${err.message}`);
      setIsSaving(false);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <div className="glass-panel widget-card">
        <div className="widget-card-header">
          <div className="widget-title">
            <Settings size={20} color="var(--accent-cyan)" />
            {isNew ? 'New Widget' : `Widget #${selectedWidgetId}: ${type}`}
          </div>

          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap', justifyContent: 'flex-end' }}>
            {widgets.map((w) => (
              <button
                key={w.id}
                className={`btn ${w.id === selectedWidgetId ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => onSelectWidget(w.id)}
                style={{ padding: '0.3rem 0.6rem', fontSize: '0.75rem' }}
              >
                #{w.id} {w.type}
              </button>
            ))}
            <button
              className={`btn ${isNew ? 'btn-primary' : 'btn-secondary'}`}
              onClick={() => onSelectWidget(null)}
              style={{ padding: '0.3rem 0.6rem', fontSize: '0.75rem' }}
            >
              <Plus size={14} /> New
            </button>
          </div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          <div className="form-group">
            <label className="form-label">Widget Type (WASM Module Name)</label>
            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <input
                    className="form-input"
                    style={{ flex: 1 }}
                    placeholder="e.g. clock, weather, news"
                    value={type}
                    onChange={(e) => setType(e.target.value)}
                />
                <span className="widget-type-badge">{type || '?'}.wasm</span>
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
            <span className="form-label" style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
              <Sliders size={14} color="var(--accent-purple)" /> Viewport Coordinates (64×64 Panel Bounds)
            </span>
            <div className="coords-grid">
              <div className="form-group">
                <label className="form-label">X Offset</label>
                <input
                  className="form-input"
                  type="number"
                  value={x}
                  onChange={(e) => setX(parseInt(e.target.value) || 0)}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Y Offset</label>
                <input
                  className="form-input"
                  type="number"
                  value={y}
                  onChange={(e) => setY(parseInt(e.target.value) || 0)}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Width</label>
                <input
                  className="form-input"
                  type="number"
                  value={width}
                  onChange={(e) => setWidth(parseInt(e.target.value) || 0)}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Height</label>
                <input
                  className="form-input"
                  type="number"
                  value={height}
                  onChange={(e) => setHeight(parseInt(e.target.value) || 0)}
                />
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
            <span className="form-label">Dynamic Key-Value Configuration</span>
            <table className="config-kv-table">
              <thead>
                <tr>
                  <th>Parameter Key</th>
                  <th>Value</th>
                  <th style={{ width: '40px' }}></th>
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
                    <td>
                      <button
                        className="btn btn-secondary"
                        style={{ padding: '0.2rem', minWidth: 'unset' }}
                        onClick={() => handleRemoveConfigPair(key)}
                      >
                        <X size={14} />
                      </button>
                    </td>
                  </tr>
                ))}
                <tr>
                  <td>
                    <input
                      className="form-input"
                      style={{ width: '100%' }}
                      placeholder="New key..."
                      value={newConfigKey}
                      onChange={(e) => setNewConfigKey(e.target.value)}
                    />
                  </td>
                  <td>
                    <input
                      className="form-input"
                      style={{ width: '100%' }}
                      placeholder="New value..."
                      value={newConfigValue}
                      onChange={(e) => setNewConfigValue(e.target.value)}
                    />
                  </td>
                  <td>
                    <button
                      className="btn btn-secondary"
                      style={{ padding: '0.2rem', minWidth: 'unset' }}
                      onClick={handleAddConfigPair}
                      disabled={!newConfigKey}
                    >
                      <Plus size={14} />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        {message && (
          <div style={{
            fontSize: '0.85rem',
            padding: '0.5rem 0.75rem',
            borderRadius: '6px',
            background: message.includes('failed') || message.includes('failed') ? 'rgba(255,71,87,0.15)' : 'rgba(0,255,136,0.15)',
            color: message.includes('failed') || message.includes('failed') ? '#ff4757' : 'var(--accent-emerald)',
            border: `1px solid ${message.includes('failed') || message.includes('failed') ? '#ff4757' : 'var(--accent-emerald)'}`
          }}>
            {message}
          </div>
        )}

        <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', marginTop: '0.5rem' }}>
          {!isNew && (
            <button className="btn btn-danger" onClick={handleDelete} disabled={isSaving}>
              <Trash2 size={16} />
              Remove Widget
            </button>
          )}
          <div style={{ flex: 1 }}></div>
          <button className="btn btn-primary" onClick={handleSave} disabled={isSaving || !type}>
            {isSaving ? <RefreshCw className="spin" size={16} /> : <Save size={16} />}
            {isNew ? 'Create Widget' : 'Replace Widget'}
          </button>
        </div>
      </div>
    </div>
  );
};
