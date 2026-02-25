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
  border-radius: 16px; /* Curved edges */
  padding: 24px;
  margin: 24px 0;
  font-family: 'Inter', -apple-system, sytem-ui, sans-serif;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 0 20px rgba(79, 70, 229, 0.15); /* Subtle brand-colored glow */
}

.zed-chat-message {
  margin-bottom: 32px;
}

.zed-chat-message:last-child {
  margin-bottom: 0;
}

.zed-chat-header {
  font-size: 0.75rem;
  font-weight: 700;
  color: #888;
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.1rem;
}

.zed-chat-content {
  color: #e5e7eb;
  line-height: 1.7;
}

.zed-chat-message.user .zed-chat-header {
  color: #60a5fa;
}

.zed-chat-message.assistant .zed-chat-header {
  color: #c084fc;
}

.zed-chat-message.tool .zed-chat-header {
  color: #34d399;
}

.zed-chat-message.tool .zed-chat-content {
  background: #252525;
  padding: 16px;
  border-radius: 12px; /* Curved tool block */
  border: 1px solid rgba(255, 255, 255, 0.05);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.875rem;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
}
</style>
