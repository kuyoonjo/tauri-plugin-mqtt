# tauri-plugin-mqtt

This plugin only works with Tauri 2.x only.

## Install

```bash
cargo add tauri-plugin-mqtt
```
```bash
npm i @kuyoonjo/tauri-plugin-mqtt
```

## Usage

### rust
```rust

tauri::Builder::default()
    .plugin(tauri_plugin_tcp::init())
    ...
```

### javascript
```javascript
import { connect, disconnect, publish, subscribe, unsubscribe } from "@kuyoonjo/tauri-plugin-mqtt";

// Server side
const id = 'unique-id';
// 增加完整url例子
await connect(id, 'mqtt://userName:passWord@test.mosquitto.org:12345?client_id=id123');
await disconnect(id);
let topic = '/tauri-plugin-mqtt';
await subscribe(id, topic, 0);
await publish(id, topic, 0, false, 'hello');
await listen((x) => {
  console.log(x.payload);
});
await unsubscribe(id, topic);
await disconnect(id);
```
#### Functions
```typescript
export async function publish(
  id: string,
  topic: String,
  qos: number,
  retain: boolean,
  payload: string | number[],
);

export async function subscribe(
  id: string,
  topic: String,
  qos: number,
);

export async function unsubscribe(
  id: string,
  topic: String,
);

export async function connect(id: string, uri: string, tlsOptions?: TlsOptions);

export async function disconnect(id: string);

export function listen(handler: EventCallback<Payload>, options?: Options);
```

#### Interfaces

```typescript
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
```

### permissions

add `"mqtt:default"` into `"permissions"` list of `src-tauri\capabilities\default.json`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  ...
  "permissions": [
    "core:default",
    ...
    "mqtt:default"
  ]
}
```

## Support

| MacOS | Linux | Windows |
| ----- | ----- | ------- |
| ✅    | ✅    | ✅      |
