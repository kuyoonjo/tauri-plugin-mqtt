<script>
  import * as mqtt from "../../../../webview-dist";
  import { onMount } from "svelte";

  let uri = "mqtt://test.mosquitto.org";
  let topic = "/tauri-plugin-mqtt";
  let message = "hello";

  async function connect() {
    try {
      if (uri.startsWith("mqtts"))
        await mqtt.connect("xxx", uri, { skipVerification: true });
      else await mqtt.connect("xxx", uri);
    } catch (e) {
      console.log({ e });
    }
  }

  async function disconnect() {
    try {
      await mqtt.disconnect("xxx");
    } catch (e) {
      console.log({ e });
    }
  }

  async function publish() {
    try {
      await mqtt.publish("xxx", topic, 0, false, message);
    } catch (e) {
      console.log({ e });
    }
  }

  async function subscribe() {
    try {
      await mqtt.subscribe("xxx", topic, 0);
    } catch (e) {
      console.log({ e });
    }
  }

  async function unsubscribe() {
    try {
      await mqtt.unsubscribe("xxx", topic);
    } catch (e) {
      console.log({ e });
    }
  }

  onMount(() => {
    mqtt.listen((x) => {
      console.log(x.payload);
    });
  });
</script>

<main class="container">
  <div class="row">
    <input placeholder="e.g. mqtt://test.mosquitto.org" bind:value={uri} />
    <button on:click={connect}> Connect </button>
    <button on:click={disconnect}> Disconnect </button>
  </div>
  <div class="row">
    <input placeholder="topic" bind:value={topic} />
  </div>
  <div class="row">
    <input placeholder="e.g. hello" bind:value={message} />
    <button on:click={subscribe}> Subscribe </button>
    <button on:click={unsubscribe}> Unsubscribe </button>
    <button on:click={publish}> Publish </button>
  </div>
</main>
