// WebAssembly + WebGPU/Canvas2D Shell for RustFlow Draw (Milestone 1)
export interface ViewportState {
  panX: number;
  panY: number;
  zoom: number;
  gridSize: number;
}

export interface NodeCard {
  id: number;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
  cornerRadius: number;
}

export interface BezierEdge {
  id: number;
  sourceId: number;
  targetId: number;
}

export class RustFlowWebShell {
  private state: ViewportState = { panX: 0, panY: 0, zoom: 1.0, gridSize: 20 };
  private nodes: NodeCard[] = [];
  private edges: BezierEdge[] = [];

  constructor(private readonly canvas: HTMLCanvasElement) {}

  public zoomAt(targetZoom: number, cursorX: number, cursorY: number): void {
    const clamped = Math.min(8.0, Math.max(0.1, targetZoom));
    const worldX = (cursorX - this.state.panX) / this.state.zoom;
    const worldY = (cursorY - this.state.panY) / this.state.zoom;
    this.state.zoom = clamped;
    this.state.panX = cursorX - worldX * clamped;
    this.state.panY = cursorY - worldY * clamped;
  }

  public addNodeCard(card: NodeCard): void {
    this.nodes.push(card);
  }

  public connectBezier(id: number, sourceId: number, targetId: number): void {
    this.edges.push({ id, sourceId, targetId });
  }
}
