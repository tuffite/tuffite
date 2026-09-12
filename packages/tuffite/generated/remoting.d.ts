// Generated from ../../framework/shell_api/remoting.d.ts. Do not edit.

export interface RemotingCapabilities {
  readonly available: boolean;
  readonly protocol: string;
  readonly videoCodecs: string[];
  readonly input: boolean;
  readonly clipboard: boolean;
  readonly reconnect: boolean;
  readonly snapshot: boolean;
}

export interface RemotingIceServer {
  readonly urls: string[];
  readonly username?: string;
  readonly credential?: string;
}

export interface RemotingStartOptions {
  /** Per-session secret supplied by the application's authenticated control plane. */
  readonly authKey: string;
  readonly iceServers: RemotingIceServer[];
  readonly maxBitrateKbps?: number;
  readonly targetFramerate?: number;
  readonly screenId?: bigint;
}

export interface RemotingSession {
  readonly id: bigint;
  readonly state: string;
}

export interface RemotingSessionDescription {
  readonly type: string;
  readonly sdp: string;
}

export interface RemotingIceCandidate {
  readonly mid: string;
  readonly mLineIndex: number;
  readonly candidate: string;
}

export interface RemotingSignal {
  readonly sequence: bigint;
  readonly description?: RemotingSessionDescription;
  readonly candidates: RemotingIceCandidate[];
}

export interface RemotingInputEvent {
  readonly kind: string;
  readonly pressed?: boolean;
  readonly usbKeycode?: number;
  readonly text?: string;
  readonly x?: number;
  readonly y?: number;
  readonly button?: string;
  readonly buttonDown?: boolean;
  readonly wheelDeltaX?: number;
  readonly wheelDeltaY?: number;
  readonly wheelTicksX?: number;
  readonly wheelTicksY?: number;
  readonly clipboardMimeType?: string;
  readonly clipboardData?: Uint8Array;
}

export interface RemotingStats {
  readonly state: string;
  readonly route: string;
  readonly transport: string;
  readonly latencyMs: number;
  readonly jitterMs: number;
  readonly availableBandwidthKbps: number;
  readonly packetLossPercent: number;
}

export interface RemotingSnapshot {
  readonly width: number;
  readonly height: number;
  readonly mimeType: string;
  readonly data: Uint8Array;
}

export declare namespace tuffite {
  export namespace remoting {
    export function capabilities(): Promise<RemotingCapabilities>;
    export function start(options: RemotingStartOptions): Promise<RemotingSession>;
    export function processSignal(id: bigint, signal: RemotingSignal): Promise<void>;
    export function takeSignals(id: bigint, afterSequence?: bigint): Promise<RemotingSignal[]>;
    export function sendInput(id: bigint, event: RemotingInputEvent): Promise<void>;
    export function stop(id: bigint): Promise<void>;
    export function stats(id: bigint): Promise<RemotingStats>;
    export function capture(screenId?: bigint): Promise<RemotingSnapshot>;
    /** Native application callers only; renderer calls are denied by routing policy. */
    export function injectInput(event: RemotingInputEvent): Promise<void>;
  }
}
