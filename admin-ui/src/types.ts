export interface SystemStatus {
  status: string;
  uptime_ms: number;
  ip: string;
  free_heap: number;
  free_psram: number;
  widget_count: number;
}

export interface WidgetInfo {
  id: number;
  type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  config: Record<string, string>;
}
