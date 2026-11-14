// WebAssembly + WebGPU/Canvas2D Bootstrap for RustFlow Draw
export interface ViewportState {
  panX: number;
  panY: number;
  zoom: number;
  gridSize: number;
}

export class RustFlowWebShell {
  private state: ViewportState = { panX: 0, panY: 0, zoom: 1.0, gridSize: 20 };

  constructor(private readonly canvas: HTMLCanvasElement) {}

  public zoomAt(targetZoom: number, cursorX: number, cursorY: number): void {
    const clamped = Math.min(8.0, Math.max(0.1, targetZoom));
    const worldX = (cursorX - this.state.panX) / this.state.zoom;
    const worldY = (cursorY - this.state.panY) / this.state.zoom;
    this.state.zoom = clamped;
    this.state.panX = cursorX - worldX * clamped;
    this.state.panY = cursorY - worldY * clamped;
  }
}
