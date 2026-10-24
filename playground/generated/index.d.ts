// Generated from Mojom v0. Do not edit.
import type { MojomStructSchema, MojomCommandEntry } from 'tuffite';
export declare const mojomTypes: Readonly<Record<string, MojomStructSchema>>;
export declare const mojomMethods: readonly MojomCommandEntry[];
export interface AuxProcessIdRequest {}
export interface AuxProcessIdResponse {
  'result': number;
}
export interface AuxReadStateRequest {}
export interface AuxReadStateResponse {
  'result': bigint;
}
export interface AuxRoundTripRequest {
  'value': MojomSample;
}
export interface AuxRoundTripResponse {
  'result': MojomSample;
}
export interface CommandsCheckJavascriptExtensionRequest {}
export interface CommandsCheckJavascriptExtensionResponse {
  'result': string;
}
export interface CommandsCheckUtilitySessionRequest {}
export interface CommandsCheckUtilitySessionResponse {
  'result': string;
}
export interface CommandsDoSomethingRequest {
  'message': string;
}
export interface CommandsDoSomethingResponse {
  'result': string;
}
export interface CommandsEchoMojomRequest {
  'value': MojomSample;
}
export interface CommandsEchoMojomResponse {
  'result': MojomSample;
}
export interface DiagnosticsProcessIdRequest {}
export interface DiagnosticsProcessIdResponse {
  'result': number;
}
export interface EchoRestrictedRequest {
  'value': MojomSample;
}
export interface EchoRestrictedResponse {
  'result': MojomSample;
}
export interface EchoRoundTripRequest {
  'value': MojomSample;
}
export interface EchoRoundTripResponse {
  'result': MojomSample;
}
export interface MojomNested {
  'label': string;
  'bytes': Uint8Array;
}
export interface MojomSample {
  'small': number;
  'large': bigint;
  'enabled': boolean;
  'visible': boolean;
  'medium': number;
  'signedValue': number;
  'fraction': number;
  'nested': MojomNested;
}
import type { MojomTransport } from 'tuffite';
export interface PlaygroundApi {
  readonly 'aux': typeof aux;
  readonly 'commands': typeof commands;
  readonly 'diagnostics': typeof diagnostics;
  readonly 'echo': typeof echo;
}
export declare function createPlaygroundApi(transport?: MojomTransport): PlaygroundApi;
export declare const aux: {
  readonly 'roundTrip': (arg0: MojomSample) => Promise<MojomSample>;
  readonly 'readState': () => Promise<bigint>;
  readonly 'processId': () => Promise<number>;
};
export declare const commands: {
  readonly 'doSomething': (arg0: string) => Promise<string>;
  readonly 'checkUtilitySession': () => Promise<string>;
  readonly 'checkJavascriptExtension': () => Promise<string>;
  readonly 'echoMojom': (arg0: MojomSample) => Promise<MojomSample>;
};
export declare const diagnostics: {
  readonly 'processId': () => Promise<number>;
};
export declare const echo: {
  readonly 'roundTrip': (arg0: MojomSample) => Promise<MojomSample>;
  readonly 'restricted': (arg0: MojomSample) => Promise<MojomSample>;
};
