<template>
  <div class="home-page">
    <!-- Hero -->
    <section class="hero">
      <div class="hero-inner">
        <!-- Left -->
        <div class="hero-left">
          <div class="badge">Windows Dev Toolchain</div>
          <h1 class="hero-title">
            <span class="brand">hudo</span><br />
            One command to install<br />all dev tools
          </h1>
          <p class="hero-tagline">
            Git, Node.js, Go, Rust, JDK, VS Code — interactive menu,
            auto PATH config, no C drive bloat.
          </p>
          <div class="hero-actions">
            <a href="/guide/quickstart" class="btn-primary">Quick Start →</a>
            <a href="/tools/" class="btn-secondary">Tool List</a>
            <a href="https://github.com/zexadev/hudo" class="btn-ghost" target="_blank">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2z"/></svg>
              GitHub
            </a>
          </div>
        </div>

        <!-- Right: Terminal -->
        <div class="hero-right">
          <div class="terminal">
            <div class="terminal-bar">
              <span class="dot red"></span>
              <span class="dot yellow"></span>
              <span class="dot green"></span>
              <span class="terminal-title">PowerShell</span>
            </div>
            <div class="terminal-body">
              <div class="line"><span class="ps">PS</span> <span class="path">C:\Users\dev</span><span class="sym">&gt;</span> <span class="cmd">irm https://zexa.cc/hudo | iex</span></div>
              <div class="line out">Downloading hudo v0.1.5...</div>
              <div class="line out success">✓ hudo installed to D:\hudo\tools\hudo.exe</div>
              <div class="line mt"><span class="ps">PS</span> <span class="path">C:\Users\dev</span><span class="sym">&gt;</span> <span class="cmd">hudo</span></div>
              <div class="line out dim">hudo v0.1.5  — Windows Dev Environment Bootstrapper</div>
              <div class="line out dim">──────────────────────────────────────────────────</div>
              <div class="line out">Select tools to install:</div>
              <div class="line out">&nbsp;</div>
              <div class="line out"><span class="sel">◉</span> Git                  <span class="ver">2.47.1</span></div>
              <div class="line out"><span class="sel">◉</span> Node.js (fnm)        <span class="ver">22.13.0</span></div>
              <div class="line out"><span class="sel">◉</span> Go                   <span class="ver">1.23.5</span></div>
              <div class="line out"><span class="sel">◉</span> Rust (rustup)        <span class="ver">1.84.0</span></div>
              <div class="line out"><span class="unsel">○</span> JDK (Eclipse Temurin)<span class="ver">21.0.5</span></div>
              <div class="line out"><span class="sel">◉</span> VS Code              <span class="ver">1.96.4</span></div>
              <div class="line out">&nbsp;</div>
              <div class="line out dim">↑↓ move  space select  enter confirm</div>
              <div class="line mt"><span class="ps">PS</span> <span class="path">C:\Users\dev</span><span class="sym">&gt;</span> <span class="cursor">█</span></div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Install command -->
    <section class="install-section">
      <p class="install-label">Quick Install — paste in PowerShell</p>
      <div class="install-box">
        <code class="install-cmd">irm https://raw.githubusercontent.com/zexadev/hudo/master/install.ps1 | iex</code>
        <button class="copy-btn" @click="copy" :class="{ copied }">
          <svg v-if="!copied" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
      </div>
    </section>

    <!-- Features -->
    <section class="features-section">
      <div class="features-grid">
        <div class="feature-card" v-for="f in features" :key="f.title">
          <div class="feature-icon" v-html="f.icon"></div>
          <h3>{{ f.title }}</h3>
          <p>{{ f.desc }}</p>
        </div>
      </div>
    </section>
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

const features = [
  {
    title: '一键安装',
    desc: '交互式多选菜单，勾选需要的工具，回车开始安装。Git、Node.js、Go、Rust、JDK、VS Code 一次搞定。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>`
  },
  {
    title: '不占 C 盘',
    desc: '首次运行选择安装盘（如 D:\\），所有工具统一安装到指定位置，系统盘永远整洁。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5"/><path d="M3 12a9 3 0 0 0 18 0"/></svg>`
  },
  {
    title: '自动配置环境变量',
    desc: '安装完成后自动写入用户 PATH，新开终端即可直接使用，无需任何手动配置。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`
  },
  {
    title: '配置档案',
    desc: '一键导出当前环境配置，换新电脑时导入即可还原，也可用于统一团队开发环境。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>`
  },
  {
    title: '自我更新',
    desc: '运行 hudo update 即可更新到最新版本，无需重新下载安装脚本。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>`
  },
  {
    title: '无需管理员权限',
    desc: '环境变量通过注册表写入当前用户，安装过程无需 UAC 提权（数据库服务注册除外）。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>`
  },
]
</script>

<style scoped>
.home-page {
  max-width: 1152px;
  margin: 0 auto;
  padding: 0 24px;
}

/* Hero */
.hero {
  padding: 80px 0 60px;
}

.hero-inner {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 48px;
  align-items: center;
}

.badge {
  display: inline-block;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--vp-c-brand-1);
  background: color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 25%, transparent);
  border-radius: 20px;
  padding: 4px 12px;
  margin-bottom: 20px;
}

.hero-title {
  font-size: clamp(2rem, 4vw, 3rem);
  font-weight: 800;
  line-height: 1.15;
  letter-spacing: -0.02em;
  color: var(--vp-c-text-1);
  margin: 0 0 20px;
}

.brand {
  color: var(--vp-c-brand-1);
}

.hero-tagline {
  font-size: 1.05rem;
  color: var(--vp-c-text-2);
  line-height: 1.7;
  margin: 0 0 32px;
  max-width: 420px;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  padding: 10px 22px;
  background: var(--vp-c-brand-1);
  color: #fff;
  border-radius: 8px;
  font-weight: 600;
  font-size: 0.95rem;
  text-decoration: none;
  transition: background 0.2s, transform 0.1s;
}
.btn-primary:hover { background: var(--vp-c-brand-2); transform: translateY(-1px); }

.btn-secondary {
  display: inline-flex;
  align-items: center;
  padding: 10px 22px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  font-weight: 600;
  font-size: 0.95rem;
  text-decoration: none;
  transition: border-color 0.2s, transform 0.1s;
}
.btn-secondary:hover { border-color: var(--vp-c-brand-1); transform: translateY(-1px); }

.btn-ghost {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  color: var(--vp-c-text-2);
  font-size: 0.9rem;
  text-decoration: none;
  border-radius: 8px;
  transition: color 0.2s;
}
.btn-ghost:hover { color: var(--vp-c-text-1); }

/* Terminal */
.hero-right {
  display: flex;
  justify-content: flex-end;
}

.terminal {
  width: 100%;
  max-width: 520px;
  background: #0d1117;
  border-radius: 12px;
  border: 1px solid #30363d;
  box-shadow: 0 24px 64px rgba(0,0,0,0.4);
  overflow: hidden;
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
}

.terminal-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 16px;
  background: #161b22;
  border-bottom: 1px solid #30363d;
}

.dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}
.dot.red    { background: #ff5f57; }
.dot.yellow { background: #febc2e; }
.dot.green  { background: #28c840; }

.terminal-title {
  margin-left: 8px;
  font-size: 0.78rem;
  color: #8b949e;
  font-family: system-ui, sans-serif;
}

.terminal-body {
  padding: 16px 20px 20px;
  font-size: 0.82rem;
  line-height: 1.7;
}

.line { display: flex; gap: 4px; flex-wrap: wrap; }
.line.mt { margin-top: 10px; }

.ps   { color: #58a6ff; }
.path { color: #79c0ff; }
.sym  { color: #8b949e; }
.cmd  { color: #e6edf3; }
.out  { color: #8b949e; padding-left: 0; }
.out.success { color: #3fb950; }
.out.dim     { color: #484f58; }
.sel   { color: #58a6ff; }
.unsel { color: #484f58; }
.ver   { color: #3fb950; margin-left: auto; padding-left: 8px; }
.cursor { color: #58a6ff; animation: blink 1s step-end infinite; }

@keyframes blink {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0; }
}

/* Install section */
.install-section {
  text-align: center;
  padding: 0 0 72px;
}

.install-label {
  font-size: 0.8rem;
  color: var(--vp-c-text-3);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: 12px;
}

.install-box {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  background: var(--vp-code-block-bg);
  border: 1px solid var(--vp-c-divider);
  border-radius: 10px;
  padding: 12px 20px;
  max-width: 100%;
}

.install-cmd {
  font-family: var(--vp-font-family-mono);
  font-size: 0.88rem;
  color: var(--vp-c-text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.copy-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--vp-c-text-3);
  padding: 0;
  transition: color 0.2s;
}
.copy-btn:hover { color: var(--vp-c-brand-1); }
.copy-btn.copied { color: var(--vp-c-green-1); }

/* Features */
.features-section {
  padding: 0 0 96px;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.feature-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 24px;
  transition: border-color 0.2s, transform 0.2s;
}
.feature-card:hover {
  border-color: var(--vp-c-brand-1);
  transform: translateY(-2px);
}

.feature-icon {
  color: var(--vp-c-brand-1);
  margin-bottom: 14px;
}

.feature-card h3 {
  font-size: 1rem;
  font-weight: 700;
  color: var(--vp-c-text-1);
  margin: 0 0 8px;
}

.feature-card p {
  font-size: 0.88rem;
  color: var(--vp-c-text-2);
  line-height: 1.65;
  margin: 0;
}

/* Responsive */
@media (max-width: 768px) {
  .hero-inner {
    grid-template-columns: 1fr;
  }
  .hero-right {
    justify-content: center;
  }
  .features-grid {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 480px) {
  .features-grid {
    grid-template-columns: 1fr;
  }
  .install-cmd {
    font-size: 0.75rem;
  }
}
</style>
