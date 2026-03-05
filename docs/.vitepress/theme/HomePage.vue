<template>
  <div class="home-page">
    <!-- Background -->
    <div class="bg-dots"></div>
    <div class="bg-glow bg-glow-1"></div>
    <div class="bg-glow bg-glow-2"></div>

    <!-- Hero -->
    <section class="hero">
      <div class="container">
        <div class="hero-badge">
          <span class="badge-dot"></span>
          Open Source — MIT License
        </div>
        <h1 class="hero-title">
          Windows 开发环境<br><span class="gradient-text">一键搞定</span>
        </h1>
        <p class="hero-desc">
          从混沌到秩序。一条命令安装、配置、管理你的整套开发工具链，<br class="hide-mobile">
          告别手动下载、PATH 冲突和注册表污染。
        </p>
        <div class="hero-install">
          <div class="install-box">
            <div class="install-shimmer"></div>
            <span class="install-prompt">PS&gt;</span>
            <code class="install-cmd">irm hudo.zexa.cc/install.ps1 | iex</code>
            <button class="install-copy" @click="copy" :title="copied ? '已复制' : '复制命令'">
              <svg v-if="!copied" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            </button>
          </div>
        </div>
        <div class="hero-actions">
          <a href="/guide/quickstart" class="btn-primary">
            快速开始
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
          </a>
          <a href="https://github.com/zexadev/hudo" class="btn-ghost" target="_blank">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
            GitHub
          </a>
        </div>
      </div>
    </section>

    <!-- Supported Tools -->
    <section class="tools-section">
      <div class="container">
        <div class="section-header">
          <p class="section-label">支持的工具</p>
          <h2 class="section-title">覆盖你的<span class="gradient-text">完整工具链</span></h2>
          <p class="section-desc">从版本控制、运行时到数据库和 IDE，18 个工具一站式管理。</p>
        </div>
        <div class="tools-grid">
          <div class="tool-card" v-for="t in tools" :key="t.name">
            <div class="tool-logo" v-html="t.svg"></div>
            <span class="tool-name">{{ t.name }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Features -->
    <section class="features-section">
      <div class="container">
        <div class="section-header">
          <p class="section-label">核心特性</p>
          <h2 class="section-title">为 Windows 开发者<span class="gradient-text">重新设计</span></h2>
          <p class="section-desc">不再需要逐个下载安装包、手动配置 PATH、处理版本冲突。<br class="hide-mobile">hudo 把一切变得简单。</p>
        </div>
        <div class="features-grid">
          <div class="feature-card" v-for="f in features" :key="f.title">
            <div class="feature-icon" v-html="f.icon"></div>
            <h3 class="feature-title">{{ f.title }}</h3>
            <p class="feature-desc">{{ f.desc }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- How it works -->
    <section class="steps-section">
      <div class="container">
        <div class="section-header">
          <p class="section-label">使用流程</p>
          <h2 class="section-title">三步开始开发</h2>
        </div>
        <div class="steps-grid">
          <div class="step-card" v-for="(s, i) in steps" :key="i">
            <div class="step-number">{{ i + 1 }}</div>
            <h3 class="step-title">{{ s.title }}</h3>
            <p class="step-desc">{{ s.desc }}</p>
            <div class="step-code" v-if="s.code">
              <code>{{ s.code }}</code>
            </div>
          </div>
          <div class="steps-connector"></div>
        </div>
      </div>
    </section>

    <!-- Terminal Demo -->
    <section class="demo-section">
      <div class="container">
        <div class="section-header">
          <p class="section-label">实际效果</p>
          <h2 class="section-title">交互式工具选择</h2>
          <p class="section-desc">运行 hudo 后，通过交互菜单勾选所需工具，一键安装到位。</p>
        </div>
        <div class="terminal-wrap">
          <div class="terminal-glow"></div>
          <div class="terminal">
            <div class="terminal-bar">
              <div class="bar-dots"><span/><span/><span/></div>
              <div class="bar-title">Administrator: PowerShell</div>
              <div class="bar-spacer"></div>
            </div>
            <div class="terminal-body">
              <div class="t-line">
                <span class="t-prompt">PS C:\Users\Dev&gt;</span>
                <span class="t-cmd">hudo</span>
              </div>
              <div class="t-output">
                <div class="t-dim">hudo v0.1.5 — The Chaos Bootstrapper</div>
                <div class="t-divider"></div>
                <div class="t-text">Select tools to install:</div>
                <div class="t-list">
                  <div class="t-item on"><span class="t-check">◉</span> Git <span class="t-ver">2.47.1</span></div>
                  <div class="t-item on"><span class="t-check">◉</span> Node.js <span class="t-ver">22.13.0</span></div>
                  <div class="t-item on"><span class="t-check">◉</span> Rust <span class="t-ver">1.84.0</span></div>
                  <div class="t-item"><span class="t-uncheck">○</span> JDK <span class="t-ver">21.0.5</span></div>
                  <div class="t-item on"><span class="t-check">◉</span> VS Code <span class="t-ver">1.96.4</span></div>
                  <div class="t-item on"><span class="t-check">◉</span> Python (uv) <span class="t-ver">0.5.x</span></div>
                  <div class="t-item"><span class="t-uncheck">○</span> MySQL <span class="t-ver">8.0</span></div>
                </div>
                <div class="t-status">
                  <div class="t-progress"><div class="t-fill"></div></div>
                  <span class="t-status-text">Installing Rust toolchain...</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Bottom CTA -->
    <section class="cta-section">
      <div class="container">
        <div class="cta-card">
          <div class="cta-glow"></div>
          <h2 class="cta-title">准备好了吗？</h2>
          <p class="cta-desc">一条命令，开启你的 Windows 开发之旅。</p>
          <div class="cta-actions">
            <a href="/guide/quickstart" class="btn-primary btn-lg">
              阅读文档
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
            </a>
            <a href="/tools/" class="btn-outline btn-lg">浏览工具列表</a>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const copied = ref(false)
function copy() {
  navigator.clipboard.writeText('irm hudo.zexa.cc/install.ps1 | iex')
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}

const tools = [
  {
    name: 'Git',
    svg: `<svg viewBox="0 0 24 24" fill="#F05032"><path d="M23.546 10.93L13.067.452a1.55 1.55 0 0 0-2.188 0L8.708 2.627l2.76 2.76a1.838 1.838 0 0 1 2.327 2.341l2.66 2.66a1.838 1.838 0 1 1-1.103 1.06l-2.48-2.48v6.53a1.838 1.838 0 1 1-1.513-.122V8.78a1.838 1.838 0 0 1-.998-2.41L7.629 3.64.452 10.816a1.55 1.55 0 0 0 0 2.188l10.48 10.48a1.55 1.55 0 0 0 2.186 0l10.43-10.43a1.55 1.55 0 0 0 0-2.123z"/></svg>`
  },
  {
    name: 'GitHub CLI',
    svg: `<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>`
  },
  {
    name: 'Node.js',
    svg: `<svg viewBox="0 0 24 24" fill="#339933"><path d="M11.998 24c-.321 0-.641-.084-.922-.247l-2.936-1.737c-.438-.245-.224-.332-.08-.383.585-.203.703-.25 1.328-.604.065-.037.151-.023.218.017l2.256 1.339a.29.29 0 0 0 .272 0l8.795-5.076a.277.277 0 0 0 .134-.238V6.921a.28.28 0 0 0-.137-.242l-8.791-5.072a.278.278 0 0 0-.271 0L3.075 6.68a.28.28 0 0 0-.138.24v10.15c0 .099.053.19.137.24l2.409 1.392c1.307.654 2.108-.116 2.108-.89V7.787c0-.142.114-.253.256-.253h1.115c.139 0 .255.112.255.253v10.021c0 1.745-.95 2.745-2.604 2.745-.508 0-.909 0-2.026-.551L2.28 18.675A1.857 1.857 0 0 1 1.36 17.07V6.921c0-.645.344-1.248.92-1.572L11.075.273a1.946 1.946 0 0 1 1.846 0l8.794 5.076c.576.324.92.927.92 1.572v10.15a1.86 1.86 0 0 1-.92 1.604l-8.795 5.078a1.834 1.834 0 0 1-.922.247z"/></svg>`
  },
  {
    name: 'Bun',
    svg: `<svg viewBox="0 0 24 24" fill="#FBF0DF"><path d="M12 22C6.477 22 2 17.523 2 12S6.477 2 12 2s10 4.477 10 10-4.477 10-10 10z"/><path d="M8.5 10.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zm7 0a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zm-7.5 3s1.5 3 4 3 4-3 4-3" fill="none" stroke="#000" stroke-width="1.5" stroke-linecap="round"/></svg>`
  },
  {
    name: 'Rust',
    svg: `<svg viewBox="0 0 24 24" fill="currentColor"><path d="M23.835 11.703l-1.008-.622-.093-.3.695-.88a.253.253 0 0 0-.031-.32l-.665-.66a.253.253 0 0 0-.318-.034l-.893.676-.3-.085-.554-1.03a.253.253 0 0 0-.276-.14l-.937.135a.253.253 0 0 0-.213.226l-.052 1.147-.268.164-.985-.468a.253.253 0 0 0-.303.072l-.635.783a.253.253 0 0 0-.014.318l.63.914-.155.285-1.107.195a.253.253 0 0 0-.208.212l-.14.934a.253.253 0 0 0 .138.28l1.023.543.04.306-.804.81a.253.253 0 0 0-.042.316l.482.85c.065.115.2.172.325.14l1.13-.25.208.24-.347 1.088a.253.253 0 0 0 .104.295l.832.524a.253.253 0 0 0 .32-.022l.818-.803.288.064.117 1.143a.253.253 0 0 0 .228.225l.968.082a.253.253 0 0 0 .268-.164l.395-1.083.296-.06.68.915a.253.253 0 0 0 .31.086l.9-.383a.253.253 0 0 0 .162-.293l-.256-1.128.235-.193 1.05.504c.12.058.265.017.337-.093l.534-.82a.253.253 0 0 0-.018-.3l-.726-.874.11-.286 1.15-.085c.131-.01.24-.103.26-.232l.155-.956a.253.253 0 0 0-.157-.278zM12.04 17.762a5.783 5.783 0 1 1 0-11.566 5.783 5.783 0 0 1 0 11.566z"/><path d="M15.06 13.728h-1.2a.29.29 0 0 1-.24-.128l-.737-1.106h-.84v1.234h-1.067V9.66h2.09c1.073 0 1.68.612 1.68 1.42 0 .596-.318 1.025-.818 1.24l.858 1.282c.085.127.01.266-.085.266zm-1.29-2.708c.419 0 .65-.236.65-.56 0-.33-.23-.56-.65-.56h-.96v1.12h.96z"/></svg>`
  },
  {
    name: 'Go',
    svg: `<svg viewBox="0 0 24 24" fill="#00ADD8"><path d="M1.811 10.231c-.047 0-.058-.023-.035-.059l.246-.315c.023-.035.081-.058.128-.058h4.172c.046 0 .058.035.035.07l-.199.303c-.023.036-.082.07-.117.07zM.047 11.306c-.047 0-.059-.023-.035-.058l.245-.316c.023-.035.082-.058.129-.058h5.328c.047 0 .07.035.058.07l-.093.28c-.012.047-.058.07-.105.07zm2.828 1.075c-.047 0-.059-.035-.035-.07l.163-.292c.023-.035.07-.07.117-.07h2.337c.047 0 .07.035.07.082l-.023.28c0 .047-.047.082-.082.082zm12.129-2.36c-.736.187-1.239.327-1.963.514-.176.046-.187.058-.34-.117-.174-.199-.303-.327-.548-.444-.737-.362-1.45-.257-2.115.175-.789.514-1.196 1.274-1.183 2.196.012.933.654 1.706 1.577 1.835.795.105 1.46-.175 1.987-.771.105-.13.198-.27.315-.434H10.47c-.245 0-.304-.152-.222-.35.152-.362.432-.97.596-1.274a.315.315 0 0 1 .292-.187h4.253c-.023.316-.023.631-.07.947a4.983 4.983 0 0 1-.958 2.29c-.841 1.11-1.94 1.8-3.33 1.986-1.145.152-2.209-.058-3.143-.72a3.963 3.963 0 0 1-1.577-2.55c-.2-1.32.105-2.55.818-3.66.783-1.226 1.858-2.023 3.213-2.37.934-.245 1.858-.222 2.744.152.654.28 1.18.713 1.588 1.284.07.105.047.163-.082.199z"/><path d="M18.872 16.218c-1.064-.024-2.034-.328-2.852-1.029a3.665 3.665 0 0 1-1.262-2.255c-.21-1.32.152-2.489.947-3.529.853-1.122 1.881-1.706 3.272-1.95 1.192-.21 2.314-.095 3.33.595.923.63 1.496 1.484 1.648 2.605.198 1.578-.257 2.863-1.344 3.962-.771.783-1.718 1.273-2.816 1.484-.304.059-.596.07-.923.117zm3.056-4.253c-.012-.166-.035-.295-.07-.432-.353-1.391-1.647-2.023-2.921-1.578-1.251.433-1.869 1.414-1.893 2.744-.012.666.269 1.274.77 1.706a2.1 2.1 0 0 0 2.372.292c1.04-.502 1.683-1.356 1.742-2.732z"/></svg>`
  },
  {
    name: 'JDK',
    svg: `<svg viewBox="0 0 24 24" fill="#ED8B00"><path d="M8.851 18.56s-.917.534.653.714c1.902.218 2.874.187 4.969-.211 0 0 .552.346 1.321.646-4.699 2.013-10.633-.118-6.943-1.149m-.575-2.627s-1.028.762.542.924c2.032.209 3.636.227 6.413-.308 0 0 .384.389.987.602-5.679 1.661-12.007.13-7.942-1.218m4.84-4.458c1.158 1.333-.304 2.533-.304 2.533s2.939-1.518 1.589-3.418c-1.261-1.772-2.228-2.652 3.007-5.688 0 0-8.216 2.051-4.292 6.573m6.924 10.083s.679.559-.747.991c-2.712.822-11.288 1.069-13.669.033-.856-.373.75-.89 1.254-.998.527-.114.828-.093.828-.093-.953-.671-6.156 1.317-2.643 1.887 9.58 1.553 17.462-.7 14.977-1.82M9.292 13.21s-4.362 1.036-1.544 1.412c1.189.159 3.561.123 5.77-.062 1.806-.152 3.618-.477 3.618-.477s-.637.272-1.098.587c-4.429 1.165-12.986.623-10.522-.568 2.082-1.006 3.776-.892 3.776-.892m7.824 4.374c4.503-2.34 2.421-4.589.968-4.285-.355.074-.515.138-.515.138s.132-.207.385-.297c2.875-1.011 5.086 2.981-.928 4.562 0 0 .07-.062.09-.118M14.401 0s2.494 2.494-2.365 6.33c-3.896 3.077-.888 4.832 0 6.836-2.274-2.053-3.943-3.858-2.824-5.539 1.644-2.469 6.197-3.665 5.189-7.627"/><path d="M9.734 23.924c4.322.277 10.959-.154 11.116-2.198 0 0-.302.775-3.572 1.391-3.688.694-8.239.613-10.937.168 0 0 .553.457 3.393.639"/></svg>`
  },
  {
    name: 'Maven',
    svg: `<svg viewBox="0 0 24 24" fill="#C71A36"><path d="M4.711 2.876L.43 10.124l4.28 7.248h2.07L2.5 10.124l4.28-7.248zm7.85 0L8.283 10.124l4.278 7.248h2.07l-4.278-7.248 4.278-7.248zm7.848 0l-4.278 7.248 4.278 7.248h2.07l-4.28-7.248 4.28-7.248z"/><rect x="2" y="18" width="20" height="3" rx="1"/></svg>`
  },
  {
    name: 'Gradle',
    svg: `<svg viewBox="0 0 24 24" fill="#02303A"><path d="M22.395 5.2a3.16 3.16 0 0 0-4.473 0c-.148.148-.3.39-.3.39L12.97 1.54s-2.132-.906-3.723.39L7.51.192 6.433 1.343l1.64 1.64s-.597.745-.894 1.342a4.42 4.42 0 0 0-.447 2.236 3.158 3.158 0 0 0 3.13 3.13c.374 0 .748-.075 1.094-.224L12 10.51s.148-.148.148-.298a.476.476 0 0 0-.148-.298L10.912 8.8a1.66 1.66 0 0 1-.8.225 1.632 1.632 0 0 1-1.565-1.565c0-.298.074-.522.224-.82.224-.447.52-.894.894-1.342L14.907 10.5a3.152 3.152 0 0 0 .298 3.872 3.16 3.16 0 0 0 4.473 0 3.16 3.16 0 0 0 0-4.473c-.745-.745-1.79-.894-2.684-.596l-.745-.745a4.78 4.78 0 0 0-.298-2.387l3.873-1.49a3.4 3.4 0 0 0 2.57.52 3.16 3.16 0 0 0 0-4.473zm-4.473 8.2a1.565 1.565 0 1 1 0 2.214 1.565 1.565 0 0 1 0-2.213zm4.175-5.987a1.565 1.565 0 1 1 0-2.213 1.565 1.565 0 0 1 0 2.213z"/><path d="M8.466 16.362c-.745.148-1.565.224-2.46.224-2.684 0-4.473-.82-4.473-1.64 0-.373.447-.82 1.268-1.118l-.82-1.268c-1.34.596-2.088 1.49-2.088 2.386 0 2.088 2.758 3.28 6.113 3.28a14.3 14.3 0 0 0 2.907-.298z"/></svg>`
  },
  {
    name: 'Python (uv)',
    svg: `<svg viewBox="0 0 24 24" fill="none"><path d="M9.585 11.692h4.328s2.432.039 2.432-2.35V5.391S16.714 3 12.304 3h-1.63C7.04 3 5.654 5.196 5.654 5.196v2.828h4.766v.706H5.654S3 8.438 3 12.688s2.317 4.063 2.317 4.063h1.384v-1.955s-.074-2.317 2.28-2.317z" fill="#366C9C"/><circle cx="8.35" cy="5.87" r=".78" fill="#FFC331"/><path d="M14.415 12.308h-4.328s-2.432-.039-2.432 2.35v3.951S7.286 21 11.696 21h1.63c3.634 0 5.02-2.196 5.02-2.196v-2.828h-4.766v-.706h4.766S21 15.562 21 11.312s-2.317-4.063-2.317-4.063h-1.384v1.955s.074 2.317-2.28 2.317z" fill="#FFC331"/><circle cx="15.65" cy="18.13" r=".78" fill="#366C9C"/></svg>`
  },
  {
    name: 'Miniconda',
    svg: `<svg viewBox="0 0 24 24" fill="#44A833"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z"/><path d="M12 6c-1.1 0-2 .67-2 1.5S10.9 9 12 9s2-.67 2-1.5S13.1 6 12 6zm-3.5 4c-.83 0-1.5.67-1.5 1.5s.67 1.5 1.5 1.5 1.5-.67 1.5-1.5S9.33 10 8.5 10zm7 0c-.83 0-1.5.67-1.5 1.5s.67 1.5 1.5 1.5 1.5-.67 1.5-1.5-.67-1.5-1.5-1.5zM10 15c-.55 0-1 .45-1 1s.45 1 1 1 1-.45 1-1-.45-1-1-1zm4 0c-.55 0-1 .45-1 1s.45 1 1 1 1-.45 1-1-.45-1-1-1z"/></svg>`
  },
  {
    name: 'MySQL',
    svg: `<svg viewBox="0 0 24 24" fill="#4479A1"><path d="M5.17 7.59c-.15 0-.24.02-.35.04v.01h.01c.07.13.18.22.27.33l.19.4.01-.01c.12-.08.18-.22.18-.43-.05-.05-.06-.12-.1-.17-.05-.07-.16-.11-.21-.17zm11.26-.26c-.47.01-.84.07-1.16.18-.09.03-.24.03-.25.15.05.05.06.12.1.18.08.13.21.3.33.39.13.1.26.21.4.3.25.16.53.25.77.4.14.09.28.21.42.31.07.05.12.13.21.16v-.02c-.05-.06-.06-.15-.11-.22-.07-.07-.14-.13-.21-.2-.21-.25-.46-.47-.73-.65-.22-.15-.7-.35-.79-.6l-.01-.01c.15-.02.33-.06.47-.1.24-.06.45-.05.69-.11.11-.03.22-.07.33-.1v-.07c-.13-.13-.22-.3-.35-.42-.34-.32-.72-.63-1.12-.89-.21-.14-.48-.23-.7-.35-.08-.04-.22-.06-.27-.13-.12-.16-.19-.36-.28-.54-.19-.38-.38-.8-.53-1.2-.11-.27-.17-.54-.31-.79-.65-1.18-1.35-1.89-2.44-2.59-.23-.15-.51-.21-.8-.29-.16-.01-.31-.01-.47-.04-.1-.04-.19-.14-.28-.2C9.79 1.6 7.97.51 7.61.97c-.22.29.34.58.53.73.14.11.32.27.43.41.07.09.09.22.15.33.14.27.27.57.44.82.09.13.18.27.29.38.07.07.18.1.2.22-.11.16-.12.39-.18.58-.29.86-.18 1.92.23 2.56.13.19.43.62.84.46.36-.14.28-.58.38-.97.03-.09.01-.15.05-.22v.02l.31.62c.23.37.64.76.98 1.02.18.14.33.37.56.46v-.02h-.01c-.07-.09-.17-.14-.25-.22-.2-.19-.41-.43-.57-.66-.23-.33-.44-.7-.62-1.08-.09-.19-.17-.4-.24-.59-.03-.07-.03-.17-.09-.21-.09.14-.22.26-.29.43-.11.28-.13.62-.17.97-.03.01-.02 0-.03.01-.26-.06-.35-.33-.45-.56-.25-.58-.29-1.51-.08-2.18.06-.17.29-.73.2-.88-.05-.15-.22-.24-.31-.36l-.39-.72c-.13-.27-.21-.55-.37-.79-.15-.22-.3-.31-.49-.46-.1-.08-.23-.23-.13-.31.03-.02.11-.04.16-.04z"/></svg>`
  },
  {
    name: 'PostgreSQL',
    svg: `<svg viewBox="0 0 24 24" fill="#4169E1"><path d="M17.128 0a10.134 10.134 0 0 0-2.755.403l-.063.02A10.922 10.922 0 0 0 12.6.258C11.422.238 10.4.524 9.594 1.01 8.97.753 7.782.397 6.473.427c-1.527.035-3.078.623-4.08 2.058-.97 1.39-1.425 3.466-1.18 6.14.073.794.207 1.618.395 2.463.277 1.243.675 2.529 1.182 3.725.498 1.176 1.09 2.22 1.838 2.982.372.379.836.72 1.382.856.547.136 1.187.062 1.687-.306.376-.276.618-.673.75-.984.043-.1.08-.198.112-.29l.009-.022c.277-.8.473-1.664.574-2.264.102.18.213.352.336.51.405.517.927.894 1.52 1.127a13.848 13.848 0 0 0-.08 1.512v.035c-.005.387.006.747.083 1.136.073.37.226.79.584 1.098.358.308.891.434 1.442.34 1.097-.187 2.07-.68 2.726-1.382.649-.693.964-1.578.95-2.476V16.4c-.002-.135-.007-.27-.017-.4l.015.004c.327.074.63.09.888.065.258-.024.472-.089.618-.145l.088-.034c.056-.02.113-.043.165-.066a3.276 3.276 0 0 0 .504-.26c.324-.207.56-.395.773-.57.424-.349.647-.588.648-.59l.018-.018.015-.02c.223-.287.232-.667.07-.987a.833.833 0 0 0-.857-.443l-.026.005c-.006.002-.014.003-.028.006l-.028.007a4.652 4.652 0 0 1-.157.034 5.612 5.612 0 0 1-.36.063 3.433 3.433 0 0 1-.363.032c-.11.003-.216-.006-.285-.022-.068-.015-.116-.03-.155-.07v-.003c.14-.51.21-1.108.206-1.69a6.73 6.73 0 0 0-.152-1.392c-.093-.4-.241-.775-.452-1.057l-.006-.008c.133-.46.232-1.028.285-1.614a8.14 8.14 0 0 0-.073-2.265c.003-.07.003-.14.002-.208v-.023a3.004 3.004 0 0 0-.025-.306 2.285 2.285 0 0 0-.15-.584c-.093-.21-.288-.47-.624-.572-.335-.103-.65-.038-.842.037a2.094 2.094 0 0 0-.44.261c.005-.156.006-.31.002-.457a6.496 6.496 0 0 0-.148-1.303 3.684 3.684 0 0 0-.39-.96 2.522 2.522 0 0 0-.498-.597c-.694-.591-1.58-.86-2.472-.948-.891-.088-1.82-.002-2.648.21-.077.02-.152.04-.225.062l-.043-.05a5.043 5.043 0 0 0-1.275-1.06C10.837.595 9.902.394 8.89.397 7.835.4 6.948.63 6.289.962h-.002C6.76.552 7.664.276 8.58.237c.914-.04 1.836.132 2.606.575A4.66 4.66 0 0 1 12.4.997c.128.147.244.304.348.468.107-.028.214-.054.326-.077.897-.184 1.812-.22 2.678-.134.866.087 1.7.348 2.354.906z"/></svg>`
  },
  {
    name: 'VS Code',
    svg: `<svg viewBox="0 0 24 24" fill="#007ACC"><path d="M17.583 2.29L9.87 9.176 5.794 5.983 4.4 6.61v10.78l1.394.627 4.076-3.193 7.713 6.886L21.6 19.77V4.23zM5.882 14.538V9.462L8.57 12zm7.082.907L9.87 12.9v-1.8l3.094-2.544v6.889z"/></svg>`
  },
  {
    name: 'PyCharm',
    svg: `<svg viewBox="0 0 24 24" fill="currentColor"><rect x="3" y="3" width="18" height="18" rx="2"/><rect x="5" y="17.5" width="7" height="1.5" fill="var(--vp-c-bg)"/><path d="M8.25 8.75h1.125c1.25 0 2.062.688 2.062 1.812 0 1.188-.875 1.876-2.125 1.876H9.25V14.5H7.812V8.312c0-.25.188-.438.438-.438zm1 2.625h.437c.438 0 .688-.25.688-.625s-.25-.625-.688-.625h-.437z" fill="var(--vp-c-bg)"/></svg>`
  },
  {
    name: 'MinGW',
    svg: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 15l3-3-3-3m5 6h4"/></svg>`
  },
  {
    name: 'Chrome',
    svg: `<svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" fill="#4285F4"/><circle cx="12" cy="12" r="4.5" fill="#fff"/><circle cx="12" cy="12" r="3" fill="#4285F4"/><path d="M12 7.5h9.5" stroke="#EA4335" stroke-width="5" opacity=".9"/><path d="M7.25 14.75L2.5 6" stroke="#34A853" stroke-width="5" opacity=".9"/><path d="M7.25 14.75l4.75-2" stroke="#FBBC05" stroke-width="5" opacity=".9"/></svg>`
  },
  {
    name: 'Claude Code',
    svg: `<svg viewBox="0 0 24 24" fill="none"><path d="M12 2l2.4 7.2H22l-6 4.8 2.4 7.2L12 16.4 5.6 21.2 8 14 2 9.2h7.6z" fill="#D97706" opacity=".9"/><path d="M12 6l1.5 4.5H18l-3.6 2.9 1.5 4.5L12 14.7l-3.9 3.2 1.5-4.5L6 10.5h4.5z" fill="#F59E0B"/></svg>`
  },
]

const features = [
  {
    title: '秒级安装',
    desc: '一条命令完成 hudo 自身安装，交互式菜单选择工具，全程自动下载、解压、配置环境变量。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>`
  },
  {
    title: '零污染隔离',
    desc: '所有工具统一安装到独立目录，不写注册表、不污染系统 PATH，卸载干净无残留。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>`
  },
  {
    title: '版本管理',
    desc: '自动检测最新版本，支持指定版本号安装。内置 GitHub API 和官方源双通道查询。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 3v12"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>`
  },
  {
    title: '镜像加速',
    desc: '内置国内镜像源支持，GitHub 下载自动走代理，大陆用户也能流畅安装。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`
  },
  {
    title: '配置档案',
    desc: '导出当前环境为 profile 文件，在新机器上一键还原。团队统一开发环境不再是难题。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>`
  },
  {
    title: '开源免费',
    desc: '基于 MIT 协议开源，Rust 编写，单文件二进制分发。社区驱动，欢迎贡献。',
    icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>`
  }
]

const steps = [
  {
    title: '安装 hudo',
    desc: '在 PowerShell 中执行一条命令，自动下载并配置 hudo。',
    code: 'irm hudo.zexa.cc/install.ps1 | iex'
  },
  {
    title: '选择工具',
    desc: '运行 hudo，在交互式菜单中勾选你需要的开发工具。',
    code: 'hudo'
  },
  {
    title: '开始开发',
    desc: '所有工具自动安装到位，环境变量已配置，打开新终端即可使用。',
    code: null
  }
]
</script>

<style scoped>
/* ── Layout ── */
.home-page {
  position: relative;
  overflow: hidden;
}

.container {
  max-width: 1120px;
  margin: 0 auto;
  padding: 0 24px;
}

/* ── Background ── */
.bg-dots {
  position: fixed;
  inset: 0;
  background-image: radial-gradient(circle, var(--vp-c-border) 1px, transparent 1px);
  background-size: 32px 32px;
  opacity: 0.4;
  pointer-events: none;
  z-index: 0;
}

.bg-glow {
  position: fixed;
  border-radius: 50%;
  filter: blur(120px);
  pointer-events: none;
  z-index: 0;
}

.bg-glow-1 {
  width: 600px;
  height: 600px;
  top: -200px;
  left: 50%;
  transform: translateX(-50%);
  background: radial-gradient(circle, rgba(59, 130, 246, 0.12) 0%, transparent 70%);
}

.bg-glow-2 {
  width: 500px;
  height: 500px;
  bottom: -100px;
  right: -100px;
  background: radial-gradient(circle, rgba(147, 51, 234, 0.08) 0%, transparent 70%);
}

/* ── Hero ── */
.hero {
  position: relative;
  z-index: 1;
  text-align: center;
  padding: 140px 0 80px;
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 18px;
  margin-bottom: 32px;
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--vp-c-text-2);
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  border-radius: 100px;
  letter-spacing: 0.02em;
}

.badge-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.6);
}

.hero-title {
  font-size: clamp(2.8rem, 6vw, 4.5rem);
  font-weight: 800;
  line-height: 1.15;
  letter-spacing: -0.03em;
  color: var(--vp-c-text-1);
  margin-bottom: 24px;
}

.gradient-text {
  background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 50%, #ec4899 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hero-desc {
  font-size: 1.15rem;
  line-height: 1.7;
  color: var(--vp-c-text-2);
  max-width: 640px;
  margin: 0 auto 40px;
}

/* ── Install Box with shimmer ── */
.hero-install {
  display: flex;
  justify-content: center;
  margin-bottom: 32px;
}

.install-box {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 14px;
  padding: 14px 20px;
  background: var(--vp-c-bg-alt);
  border: 1px solid transparent;
  border-radius: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 0.9rem;
  overflow: hidden;
  z-index: 1;
}

.install-shimmer {
  position: absolute;
  inset: 0;
  border-radius: 12px;
  padding: 1px;
  background: linear-gradient(
    135deg,
    transparent 20%,
    rgba(59, 130, 246, 0.4) 40%,
    rgba(139, 92, 246, 0.4) 50%,
    rgba(236, 72, 153, 0.3) 60%,
    transparent 80%
  );
  background-size: 300% 300%;
  animation: shimmer 6s ease-in-out infinite;
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  z-index: -1;
}

@keyframes shimmer {
  0% { background-position: 100% 0%; }
  50% { background-position: 0% 100%; }
  100% { background-position: 100% 0%; }
}

.install-prompt {
  color: var(--vp-c-text-3);
  font-weight: 600;
  user-select: none;
}

.install-cmd {
  color: var(--vp-c-text-1);
  font-weight: 500;
}

.install-copy {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid var(--vp-c-border);
  background: var(--vp-c-bg);
  cursor: pointer;
  color: var(--vp-c-text-3);
  transition: all 0.15s;
}
.install-copy:hover {
  color: var(--vp-c-text-1);
  border-color: var(--vp-c-text-3);
}

/* Hero Buttons */
.hero-actions {
  display: flex;
  justify-content: center;
  gap: 14px;
  flex-wrap: wrap;
}

.btn-primary, .btn-ghost, .btn-outline {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0 28px;
  height: 48px;
  border-radius: 10px;
  font-weight: 600;
  font-size: 0.95rem;
  text-decoration: none;
  transition: all 0.2s;
  cursor: pointer;
}

.btn-primary {
  background: var(--vp-c-brand-1);
  color: #fff;
  border: 1px solid transparent;
}
.btn-primary:hover {
  filter: brightness(1.15);
  transform: translateY(-1px);
  box-shadow: 0 4px 20px rgba(59, 130, 246, 0.3);
}

.btn-ghost {
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  border: 1px solid var(--vp-c-border);
}
.btn-ghost:hover {
  border-color: var(--vp-c-text-3);
  background: var(--vp-c-bg-alt);
}

.btn-outline {
  background: transparent;
  color: var(--vp-c-text-1);
  border: 1px solid var(--vp-c-border);
}
.btn-outline:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.btn-lg {
  height: 52px;
  padding: 0 32px;
  font-size: 1rem;
}

/* ── Tools Section — Logo Card Grid ── */
.tools-section {
  position: relative;
  z-index: 1;
  padding: 40px 0 80px;
}

.section-label {
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--vp-c-brand-1);
  margin-bottom: 20px;
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 14px;
  max-width: 900px;
  margin: 0 auto;
}

.tool-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 20px 12px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  border-radius: 14px;
  transition: all 0.25s;
  cursor: default;
}

.tool-card:hover {
  border-color: rgba(59, 130, 246, 0.5);
  transform: translateY(-3px);
  box-shadow:
    0 0 0 1px rgba(59, 130, 246, 0.1),
    0 8px 24px -8px rgba(59, 130, 246, 0.15);
}

.dark .tool-card:hover {
  box-shadow:
    0 0 0 1px rgba(96, 165, 250, 0.15),
    0 8px 24px -8px rgba(96, 165, 250, 0.2);
}

.tool-logo {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tool-logo :deep(svg) {
  width: 100%;
  height: 100%;
}

.tool-name {
  font-size: 0.78rem;
  font-weight: 500;
  color: var(--vp-c-text-2);
  text-align: center;
  line-height: 1.2;
}

/* ── Features with gradient border glow ── */
.features-section {
  position: relative;
  z-index: 1;
  padding: 80px 0;
}

.section-header {
  text-align: center;
  margin-bottom: 56px;
}

.section-title {
  font-size: clamp(1.8rem, 3.5vw, 2.5rem);
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--vp-c-text-1);
  margin-bottom: 16px;
  line-height: 1.2;
}

.section-desc {
  font-size: 1.05rem;
  line-height: 1.7;
  color: var(--vp-c-text-2);
  max-width: 560px;
  margin: 0 auto;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.feature-card {
  position: relative;
  padding: 32px 28px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  border-radius: 16px;
  transition: all 0.3s;
  overflow: hidden;
}

.feature-card::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 16px;
  padding: 1px;
  background: linear-gradient(
    135deg,
    rgba(59, 130, 246, 0) 0%,
    rgba(59, 130, 246, 0) 40%,
    rgba(59, 130, 246, 0) 100%
  );
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  transition: all 0.3s;
  pointer-events: none;
}

.feature-card:hover {
  border-color: transparent;
  transform: translateY(-3px);
  box-shadow:
    0 0 0 1px rgba(59, 130, 246, 0.1),
    0 16px 48px -12px rgba(0, 0, 0, 0.12);
}

.feature-card:hover::before {
  background: linear-gradient(
    135deg,
    rgba(59, 130, 246, 0.6) 0%,
    rgba(139, 92, 246, 0.5) 50%,
    rgba(236, 72, 153, 0.4) 100%
  );
}

.dark .feature-card:hover {
  box-shadow:
    0 0 0 1px rgba(96, 165, 250, 0.15),
    0 16px 48px -12px rgba(0, 0, 0, 0.5);
}

.feature-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-border);
  color: var(--vp-c-brand-1);
  margin-bottom: 20px;
  position: relative;
}

.feature-title {
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--vp-c-text-1);
  margin-bottom: 10px;
  position: relative;
}

.feature-desc {
  font-size: 0.9rem;
  line-height: 1.65;
  color: var(--vp-c-text-2);
  position: relative;
}

/* ── Steps ── */
.steps-section {
  position: relative;
  z-index: 1;
  padding: 80px 0;
}

.steps-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
  position: relative;
}

.steps-connector {
  position: absolute;
  top: 36px;
  left: calc(16.67% + 24px);
  right: calc(16.67% + 24px);
  height: 2px;
  background: linear-gradient(to right, var(--vp-c-brand-1), var(--vp-c-brand-3));
  opacity: 0.3;
  z-index: 0;
}

.step-card {
  position: relative;
  z-index: 1;
  text-align: center;
  padding: 32px 24px;
}

.step-number {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 20px;
  border-radius: 50%;
  background: var(--vp-c-bg);
  border: 2px solid var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
  font-size: 1.1rem;
  font-weight: 700;
}

.step-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--vp-c-text-1);
  margin-bottom: 10px;
}

.step-desc {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  line-height: 1.6;
  margin-bottom: 16px;
}

.step-code {
  display: inline-flex;
  padding: 8px 16px;
  background: var(--vp-c-bg-alt);
  border: 1px solid var(--vp-c-border);
  border-radius: 8px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.82rem;
  color: var(--vp-c-brand-1);
}

/* ── Demo Terminal with glow ── */
.demo-section {
  position: relative;
  z-index: 1;
  padding: 80px 0 100px;
}

.terminal-wrap {
  max-width: 680px;
  margin: 0 auto;
  position: relative;
}

.terminal-glow {
  position: absolute;
  inset: -40px;
  background: radial-gradient(
    ellipse at center,
    rgba(59, 130, 246, 0.08) 0%,
    rgba(139, 92, 246, 0.05) 40%,
    transparent 70%
  );
  border-radius: 50%;
  filter: blur(40px);
  pointer-events: none;
  z-index: -1;
}

.dark .terminal-glow {
  background: radial-gradient(
    ellipse at center,
    rgba(59, 130, 246, 0.12) 0%,
    rgba(139, 92, 246, 0.08) 40%,
    transparent 70%
  );
}

.terminal {
  position: relative;
  background: #0a0a0f;
  border-radius: 14px;
  border: 1px solid #1e1e2e;
  overflow: hidden;
  box-shadow:
    0 0 0 1px rgba(255, 255, 255, 0.04),
    0 24px 64px -16px rgba(0, 0, 0, 0.55),
    0 0 80px -20px rgba(59, 130, 246, 0.1);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.terminal-bar {
  display: flex;
  align-items: center;
  padding: 14px 18px;
  background: #141420;
  border-bottom: 1px solid #1e1e2e;
}

.bar-dots {
  display: flex;
  gap: 7px;
}
.bar-dots span {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  background: #2a2a3a;
}
.bar-dots span:first-child { background: #ff5f57; }
.bar-dots span:nth-child(2) { background: #ffbd2e; }
.bar-dots span:last-child { background: #28c840; }

.bar-title {
  flex: 1;
  text-align: center;
  font-size: 0.72rem;
  color: #555;
  font-family: var(--vp-font-family-base);
}

.bar-spacer { width: 52px; }

.terminal-body {
  padding: 24px 28px 28px;
  color: #c9d1d9;
  font-size: 0.84rem;
  line-height: 1.65;
}

.t-line {
  display: flex;
  gap: 10px;
  margin-bottom: 18px;
}

.t-prompt { color: #a78bfa; font-weight: 600; white-space: nowrap; }
.t-cmd { color: #fff; }

.t-output {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.t-dim { color: #484858; }
.t-text { color: #c9d1d9; }
.t-divider { height: 1px; background: #1e1e2e; margin: 6px 0; }

.t-list {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin: 8px 0;
}

.t-item {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #6e6e80;
}
.t-item.on { color: #e2e8f0; }

.t-check { color: #3b82f6; }
.t-uncheck { color: #3a3a4a; }
.t-ver { margin-left: auto; color: #3a3a4a; font-size: 0.75rem; }

.t-status {
  margin-top: 16px;
  display: flex;
  align-items: center;
  gap: 14px;
}

.t-progress {
  flex: 1;
  height: 3px;
  background: #1e1e2e;
  border-radius: 2px;
  overflow: hidden;
}

.t-fill {
  width: 60%;
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  border-radius: 2px;
}

.t-status-text {
  font-size: 0.72rem;
  color: #555;
  white-space: nowrap;
}

/* ── CTA ── */
.cta-section {
  position: relative;
  z-index: 1;
  padding: 0 0 120px;
}

.cta-card {
  position: relative;
  text-align: center;
  padding: 72px 40px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  border-radius: 20px;
  overflow: hidden;
}

.cta-glow {
  position: absolute;
  top: -50%;
  left: 50%;
  transform: translateX(-50%);
  width: 80%;
  height: 200%;
  background: radial-gradient(ellipse, rgba(59, 130, 246, 0.06) 0%, transparent 60%);
  pointer-events: none;
}

.cta-title {
  font-size: clamp(1.8rem, 3vw, 2.5rem);
  font-weight: 700;
  color: var(--vp-c-text-1);
  margin-bottom: 12px;
  position: relative;
}

.cta-desc {
  font-size: 1.1rem;
  color: var(--vp-c-text-2);
  margin-bottom: 36px;
  position: relative;
}

.cta-actions {
  display: flex;
  justify-content: center;
  gap: 14px;
  position: relative;
  flex-wrap: wrap;
}

/* ── Responsive ── */
@media (max-width: 768px) {
  .hero { padding: 100px 0 60px; }

  .hide-mobile { display: none; }

  .install-box {
    font-size: 0.8rem;
    padding: 12px 16px;
  }

  .tools-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }

  .tool-card { padding: 14px 8px; }
  .tool-logo { width: 28px; height: 28px; }
  .tool-name { font-size: 0.7rem; }

  .features-grid {
    grid-template-columns: 1fr;
    gap: 14px;
  }

  .steps-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .steps-connector { display: none; }

  .step-card { padding: 24px 16px; }

  .terminal-body { padding: 18px 20px; }

  .cta-card { padding: 48px 24px; }

  .btn-primary, .btn-ghost, .btn-outline {
    height: 44px;
    padding: 0 22px;
    font-size: 0.9rem;
  }

  .btn-lg {
    height: 48px;
    padding: 0 26px;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .tools-grid {
    grid-template-columns: repeat(4, 1fr);
  }

  .features-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
