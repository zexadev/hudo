<template>
  <div class="install-block">
    <p class="install-label">快速安装</p>
    <div class="install-code">
      <code>irm https://raw.githubusercontent.com/zexadev/hudo/master/install.ps1 | iex</code>
      <button class="copy-btn" @click="copy" :class="{ copied }">
        <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const copied = ref(false)

function copy() {
  navigator.clipboard.writeText('irm https://raw.githubusercontent.com/zexadev/hudo/master/install.ps1 | iex')
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}
</script>

<style scoped>
.install-block {
  margin-top: 24px;
  text-align: center;
}

.install-label {
  font-size: 0.85rem;
  color: var(--vp-c-text-3);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.install-code {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  background: var(--vp-code-block-bg);
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  padding: 10px 16px;
}

.install-code code {
  font-family: var(--vp-font-family-mono);
  font-size: 0.88rem;
  color: var(--vp-c-text-1);
  white-space: nowrap;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--vp-c-text-3);
  cursor: pointer;
  transition: color 0.2s;
  background: none;
  border: none;
  padding: 0;
}

.copy-btn:hover { color: var(--vp-c-brand); }
.copy-btn.copied { color: var(--vp-c-green); }
</style>
