import { invoke } from '@tauri-apps/api/core';
import { EventCallback, Options, listen as _listen } from '@tauri-apps/api/event';
import { Buffer } from 'buffer';

/**
 * 
 * @param id A unique ID
 * @param topic The topic to subscribe to
 * @param qos The QoS level
 * @param retain Whether the message should be retained
 * @param payload The payload
 */
export async function publish(
  id: string,
  topic: String,
  qos: number,
  retain: boolean,
  payload: string | number[],
) {
  await invoke('plugin:mqtt|publish', {
    id, topic, qos, retain, payload: typeof payload === 'string' ? Array.from(Buffer.from(payload)) : payload,
  });
}

/**
 * 
 * @param id A unique ID
 * @param topic The topic to subscribe to
 * @param qos The QoS level
 */
export async function subscribe(
  id: string,
  topic: String,
  qos: number,
) {
  await invoke('plugin:mqtt|subscribe', {
    id, topic, qos,
  });
}

/**
 * 
 * @param id A unique ID
 * @param topic The topic to subscribe to
 */
export async function unsubscribe(
  id: string,
  topic: String,
) {
  await invoke('plugin:mqtt|unsubscribe', {
    id, topic,
  });
}


/**
 * 
 * @param id A unique ID
 * @param uri e.g. mqtt://test.mosquitto.org
 */
export async function connect(id: string, uri: string, tlsOptions?: TlsOptions) {
  await invoke('plugin:mqtt|connect', {
    id, uri, tlsOptions,
  });
}

/**
 * 
 * @param id A unique ID
 */
export async function disconnect(id: string) {
  await invoke('plugin:mqtt|disconnect', {
    id
  });
}

export interface TlsOptions {
  skipVerification?: boolean;
  ca?: number[];
  alpn?: number[][];
  client_cert?: number[];
  client_key?: number[];
}

export interface Payload {
  id: string;
  event: {
    connect?: [];
    disconnect?: [];
    message?: {
      dup: boolean;
      qos: 0 | 1 | 2;
      retain: boolean;
      topic: string;
      pkid: number;
      payload: number[];
    };
  };
}

export function listen(handler: EventCallback<Payload>, options?: Options) {
  return _listen('plugin://mqtt', handler, options);
}
