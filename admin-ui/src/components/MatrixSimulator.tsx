import React from 'react';
import type { WidgetInfo } from '../types';
import { Monitor, Cpu, Layers } from 'lucide-react';

interface MatrixSimulatorProps {
  widgets: WidgetInfo[];
  selectedWidgetId: number | null;
  onSelectWidget: (id: number) => void;
}

export const MatrixSimulator: React.FC<MatrixSimulatorProps> = ({
  widgets,
  selectedWidgetId,
  onSelectWidget,
}) => {
  return (
    <div className="glass-panel matrix-container">
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Monitor size={18} color="var(--accent-cyan)" />
          <h3 style={{ fontFamily: 'var(--font-heading)', fontSize: '1rem' }}>64×64 LED Matrix Live Grid</h3>
        </div>
        <span style={{ fontSize: '0.75rem', fontFamily: 'var(--font-mono)', color: 'var(--text-muted)' }}>
          HUB75 (1/32 scan)
        </span>
      </div>

      <div className="matrix-canvas-wrapper">
        <div className="matrix-grid-overlay" />
        {widgets.map((w) => {
          const isSelected = selectedWidgetId === w.id;
          // Scale 64x64 to 320x320 px canvas (scale factor = 5)
          const left = w.x * 5;
          const top = w.y * 5;
          const width = w.width * 5;
          const height = w.height * 5;

          return (
            <div
              key={w.id}
              className={`matrix-widget-box ${isSelected ? 'active' : ''}`}
              style={{
                left: `${left}px`,
                top: `${top}px`,
                width: `${width}px`,
                height: `${height}px`,
              }}
              onClick={() => onSelectWidget(w.id)}
            >
              <div className="widget-box-label">{w.type}</div>
              <div className="widget-box-coords">
                #{w.id} ({w.x},{w.y}) {w.width}×{w.height}
              </div>
            </div>
          );
        })}
      </div>

      <div style={{ display: 'flex', gap: '1rem', width: '100%', justifyContent: 'center', marginTop: '0.5rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
          <Layers size={14} color="var(--accent-cyan)" /> {widgets.length} Loaded Viewports
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
          <Cpu size={14} color="var(--accent-emerald)" /> Core 1 Refresh Active
        </div>
      </div>
    </div>
  );
};
