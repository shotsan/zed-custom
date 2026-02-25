<template>
  <div class="zed-chat-container">
    <div class="zed-chat-message user">
      <div class="zed-chat-header">User</div>
      <div class="zed-chat-content">
        <slot name="user"></slot>
      </div>
    </div>
    <div class="zed-chat-message assistant">
      <div class="zed-chat-header">Zed Agent</div>
      <div class="zed-chat-content">
        <slot name="assistant"></slot>
      </div>
    </div>
    <div v-if="$slots.output" class="zed-chat-message tool">
      <div class="zed-chat-header">Tool Output</div>
      <div class="zed-chat-content">
        <slot name="output"></slot>
      </div>
    </div>
  </div>
</template>

<style scoped>
.zed-chat-container {
  background-color: #1e1e1e;
  border-radius: 8px;
  padding: 16px;
  margin: 20px 0;
  font-family: 'Inter', -apple-system, sytem-ui, sans-serif;
  border: 1px solid #333;
}

.zed-chat-message {
  margin-bottom: 24px;
}

.zed-chat-message:last-child {
  margin-bottom: 0;
}

.zed-chat-header {
  font-size: 0.8rem;
  font-weight: 600;
  color: #888;
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05rem;
}

.zed-chat-content {
  color: #e0e0e0;
  line-height: 1.6;
}

.zed-chat-message.user .zed-chat-header {
  color: #3b82f6;
}

.zed-chat-message.assistant .zed-chat-header {
  color: #a855f7;
}

.zed-chat-message.tool .zed-chat-header {
  color: #10b981;
}

.zed-chat-message.tool .zed-chat-content {
  background: #252525;
  padding: 12px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
}
</style>
