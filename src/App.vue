<template>
  <div id="app">
    <h1>FIFO Queue Demo</h1>
    <div class="container">
      <div class="queue-container">
        <h2>Queue</h2>
        <div id="queue">
          <div v-for="packet in packets" :key="packet.id" class="packet">
            Packet {{ packet.id }}: {{ packet.content }}
          </div>
        </div>
      </div>
      <div class="controls-container">
        <h2>Controls</h2>
        <label for="producerCount">Number of Producers:</label>
        <input type="number" id="producerCount" v-model.number="producerCount" min="1" />
        <button @click="updateProducerCount">Set Producers</button>
      </div>
    </div>
  </div>
</template>
<script>
import { onMounted, ref } from 'vue';
import { getCurrent } from "@tauri-apps/api/webview";
import { invoke } from '@tauri-apps/api/core';

export default {
  name: 'App',
  setup() {
    const packets = ref([]);
    const producerCount = ref(3);

    onMounted(() => {
      getCurrent().listen('packet_received', ({event, packet}) => {
        console.log("Packet received: ", packet.payload);
        packets.value.push(event.payload.id);
      });

      getCurrent().listen('packet_processed', ({event, packet}) => {
        console.log("Packet processed: ", event.payload);
        packets.value.pop(packet.payload);
      });
    });

    const updateProducerCount = async () => {
      await invoke('set_producer_count', { count: producerCount.value });
    };

    return { packets, producerCount, updateProducerCount };
  }
};
</script>

<style>
.container {
  display: flex;
}

.queue-container {
  flex: 1;
  padding: 10px;
}

.controls-container {
  flex: 1;
  padding: 10px;
  border-left: 1px solid #ccc;
}

.packet {
  padding: 10px;
  margin: 5px;
  border: 1px solid #000;
  border-radius: 5px;
  background-color: #f0f0f0;
}
</style>