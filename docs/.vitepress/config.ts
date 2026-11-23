import { defineConfig, type HeadConfig } from 'vitepress'

const SITE = 'https://hudo.zexa.cc'
const SITE_TITLE = 'hudo - Windows 开发环境一键引导工具'
const SITE_DESC = '一条命令安装 Git、Node.js、Rust、Go、JDK、Python 等 26 款开发工具，免管理员权限，自动配置环境变量，支持国内镜像加速。'

// 生成与 sitemap 完全一致的页面绝对 URL（canonical 必须与 sitemap/内链/线上服务形态一致；
// Cloudflare Pages 把 .html 308 到无扩展名 URL，故全站统一 clean URL 形态 + cleanUrls: true）
function pageUrl(page: string): string {
  const path = page.replace(/\.md$/, '')
  if (path === 'index') return `${SITE}/`
  if (path.endsWith('/index')) return `${SITE}/${path.slice(0, -'index'.length)}`
  return `${SITE}/${path}`
}

// 面包屑分区：有真实索引页的分区才给中间层级
const SECTIONS: Record<string, { name: string; url: string }> = {
  tools: { name: '工具列表', url: `${SITE}/tools/` },
  blog: { name: '博客', url: `${SITE}/blog/` },
}

export default defineConfig({
  title: 'hudo',
  description: SITE_DESC,
  lang: 'zh-CN',
  // 与 Cloudflare Pages 的 URL 归一化行为对齐（平台把 .html 308 到无扩展名 URL）
  cleanUrls: true,
  // sitemap lastmod 与页面"最后更新"时间均取 git 提交时间（新鲜度信号）
  lastUpdated: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    // 首页入场动画的隐藏基态以 html.js 为门控：无 JS/爬虫/慢网首帧下内容默认可见
    ['script', {}, 'document.documentElement.classList.add("js")'],
    // SEO meta
    ['meta', { name: 'keywords', content: 'hudo, Windows开发环境, 一键安装, 开发工具, Git安装, Node.js安装, JDK安装, Rust安装, Python环境, Go开发环境, MySQL安装, 包管理器, 免管理员, dev tools, Windows development' }],
    ['meta', { name: 'author', content: 'Zexa' }],
    // Open Graph 站点级标签；og:title/og:description/og:url/canonical 逐页在 transformHead 输出
    ['meta', { property: 'og:site_name', content: 'hudo' }],
    // 社交爬虫不渲染 SVG，分享卡图必须用位图
    ['meta', { property: 'og:image', content: `${SITE}/og-image.png` }],
    ['meta', { property: 'og:image:width', content: '1200' }],
    ['meta', { property: 'og:image:height', content: '630' }],
    ['meta', { property: 'og:image:alt', content: 'hudo - Windows 开发环境一键引导工具' }],
    ['meta', { property: 'og:locale', content: 'zh_CN' }],
    // Twitter Card（title/description 缺省时 X 会回退读 OG 标签）
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:image', content: `${SITE}/og-image.png` }],
    // Schema.org 结构化数据（SoftwareApplication 是 Google 仍支持的富结果类型）
    ['script', { type: 'application/ld+json' }, JSON.stringify({
      '@context': 'https://schema.org',
      '@type': 'SoftwareApplication',
      'name': 'hudo',
      'applicationCategory': 'DeveloperApplication',
      'operatingSystem': 'Windows',
      'description': 'Windows 开发环境一键引导工具：一条命令安装 26 款开发工具（Git/Node.js/Rust/Go/JDK/Python/MySQL 等），免管理员权限，自动配置环境变量，支持国内镜像与环境档案一键还原。',
      'url': SITE,
      'downloadUrl': 'https://github.com/zexadev/hudo/releases',
      'author': {
        '@type': 'Organization',
        'name': 'Zexa',
        'url': 'https://zexa.cc'
      },
      'license': 'https://opensource.org/licenses/MIT',
      'offers': {
        '@type': 'Offer',
        'price': '0',
        'priceCurrency': 'CNY'
      }
    })],
  ],

  sitemap: {
    hostname: 'https://hudo.zexa.cc',
  },

  themeConfig: {
    siteTitle: 'hudo',

    nav: [
      { text: '指南', link: '/guide/what-is-hudo' },
      { text: '工具列表', link: '/tools/' },
      { text: '博客', link: '/blog/' },
      {
        text: 'v0.4.1',
        items: [
          { text: '更新日志', link: '/changelog' },
          { text: 'GitHub', link: 'https://github.com/zexadev/hudo' },
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: '开始',
          items: [
            { text: '什么是 hudo？', link: '/guide/what-is-hudo' },
            { text: '安装', link: '/guide/install' },
            { text: '快速上手', link: '/guide/quickstart' },
          ]
        },
        {
          text: '进阶',
          items: [
            { text: '配置文件', link: '/guide/config' },
            { text: '配置档案', link: '/guide/profile' },
            { text: '自我更新', link: '/guide/update' },
          ]
        }
      ],
      '/tools/': [
        {
          text: '工具',
          items: [
            { text: '总览', link: '/tools/' },
            { text: 'Git', link: '/tools/git' },
            { text: 'GitHub CLI', link: '/tools/gh' },
            { text: 'Node.js', link: '/tools/nodejs' },
            { text: 'fnm', link: '/tools/fnm' },
            { text: 'Bun', link: '/tools/bun' },
            { text: 'Rust', link: '/tools/rust' },
            { text: 'Go', link: '/tools/go' },
            { text: 'JDK', link: '/tools/jdk' },
            { text: 'Maven', link: '/tools/maven' },
            { text: 'Gradle', link: '/tools/gradle' },
            { text: 'Python (uv)', link: '/tools/python' },
            { text: 'Miniconda', link: '/tools/miniconda' },
            { text: '.NET SDK', link: '/tools/dotnet' },
            { text: 'MySQL', link: '/tools/mysql' },
            { text: 'PostgreSQL', link: '/tools/pgsql' },
            { text: 'Redis', link: '/tools/redis' },
            { text: 'VS Code', link: '/tools/vscode' },
            { text: 'PyCharm', link: '/tools/pycharm' },
            { text: 'IntelliJ IDEA', link: '/tools/idea' },
            { text: 'MinGW', link: '/tools/mingw' },
            { text: '7-Zip', link: '/tools/7zip' },
            { text: 'PowerShell 7', link: '/tools/pwsh' },
            { text: 'PowerToys', link: '/tools/powertoys' },
            { text: 'Oh My Posh', link: '/tools/omp' },
            { text: 'Google Chrome', link: '/tools/chrome' },
            { text: 'Claude Code', link: '/tools/claude-code' },
          ]
        }
      ],
      '/blog/': [
        {
          text: '博客',
          items: [
            { text: '全部文章', link: '/blog/' },
            { text: '程序员换电脑迁移指南', link: '/blog/windows-pc-migration-guide' },
            { text: 'Windows 终端美化', link: '/blog/windows-terminal-beautify' },
            { text: '终端图标方框乱码修复', link: '/blog/windows-nerd-font-fix' },
            { text: '环境变量彻底讲清', link: '/blog/windows-env-variables' },
            { text: 'PowerShell 7 升级指南', link: '/blog/windows-powershell7-guide' },
            { text: 'Java 环境全家桶搭建', link: '/blog/windows-java-env-setup' },
            { text: '7z 命令行速查', link: '/blog/windows-7zip-cli' },
            { text: '.NET 安装与多版本共存', link: '/blog/windows-dotnet-install' },
            { text: 'PowerToys 值得装吗', link: '/blog/windows-powertoys-guide' },
            { text: 'Windows Git 一键安装', link: '/blog/windows-git-install' },
            { text: 'Windows Node.js 环境搭建', link: '/blog/windows-nodejs-install' },
            { text: 'Windows JDK 一键安装', link: '/blog/windows-jdk-install' },
            { text: 'Windows MySQL 一键部署', link: '/blog/windows-mysql-install' },
            { text: 'Windows Rust 安装不踩坑', link: '/blog/windows-rust-install' },
            { text: 'Windows Redis 安装 + 开机自启', link: '/blog/windows-redis-install' },
            { text: 'uv 一键管理 Python 环境', link: '/blog/windows-python-uv-install' },
            { text: 'Miniconda 轻量 Python 环境搭建', link: '/blog/windows-miniconda-install' },
            { text: 'MinGW-w64 一键安装', link: '/blog/windows-mingw-install' },
            { text: 'Maven 安装配置一条龙', link: '/blog/windows-maven-install' },
            { text: '20 分钟配好完整开发环境', link: '/blog/windows-dev-environment-setup' },
            { text: 'PyCharm 社区版安装配置', link: '/blog/windows-pycharm-install' },
            { text: 'VS Code 便携版安装', link: '/blog/windows-vscode-install' },
            { text: 'Bun 快速上手', link: '/blog/windows-bun-install' },
            { text: '包管理器横评', link: '/blog/windows-package-manager-compare' },
            { text: 'Go 开发环境配置', link: '/blog/windows-go-install' },
            { text: 'Gradle 安装与选择', link: '/blog/windows-gradle-install' },
            { text: 'PostgreSQL 快速安装', link: '/blog/windows-postgresql-install' },
            { text: 'Claude Code AI 编程助手', link: '/blog/windows-claude-code-install' },
            { text: 'GitHub CLI 实用命令', link: '/blog/windows-gh-install' },
            { text: '一键迁移开发环境', link: '/blog/windows-dev-env-migration' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/zexadev/hudo' }
    ],

    footer: {
      message: '基于 MIT 协议发布',
      copyright: 'Copyright © 2025-2026 Zexa'
    },

    search: {
      provider: 'local'
    },

    editLink: {
      pattern: 'https://github.com/zexadev/hudo/edit/master/docs/:path',
      text: '在 GitHub 上编辑此页'
    }
  },

  transformHead(context) {
    const head: HeadConfig[] = []
    const { frontmatter } = context.pageData
    // page 在 TransformContext 顶层（如 "tools/git.md"），pageData 里没有这个字段
    const page = context.page
    const pagePath = page.replace(/\.md$/, '').replace(/\/index$/, '')
    const url = pageUrl(page)
    const title = (frontmatter.title as string) || SITE_TITLE
    const desc = (frontmatter.description as string) || SITE_DESC
    const isBlogPost = pagePath.startsWith('blog/') && pagePath !== 'blog'

    // canonical + 逐页 OG：合并 .html/尾斜杠等 URL 变体信号，社交分享显示每页真实标题
    head.push(['link', { rel: 'canonical', href: url }])
    head.push(['meta', { property: 'og:title', content: title }])
    head.push(['meta', { property: 'og:description', content: desc }])
    head.push(['meta', { property: 'og:url', content: url }])
    head.push(['meta', { property: 'og:type', content: isBlogPost ? 'article' : 'website' }])

    // 面包屑（首页不输出；有真实索引页的分区给三级，其余两级）
    if (page !== 'index.md') {
      const shortTitle = title.split(/\s+[-—|]\s+/)[0]
      const section = SECTIONS[pagePath.split('/')[0]]
      const items: Record<string, unknown>[] = [
        { '@type': 'ListItem', 'position': 1, 'name': '首页', 'item': `${SITE}/` },
      ]
      if (section && pagePath.includes('/')) {
        items.push({ '@type': 'ListItem', 'position': 2, 'name': section.name, 'item': section.url })
        items.push({ '@type': 'ListItem', 'position': 3, 'name': shortTitle })
      } else {
        items.push({ '@type': 'ListItem', 'position': 2, 'name': shortTitle })
      }
      head.push(['script', { type: 'application/ld+json' }, JSON.stringify({
        '@context': 'https://schema.org',
        '@type': 'BreadcrumbList',
        'itemListElement': items,
      })])
    }

    // 博客文章 → Article schema
    if (isBlogPost) {
      const article: Record<string, unknown> = {
        '@context': 'https://schema.org',
        '@type': 'Article',
        'headline': title,
        'description': desc,
        'author': { '@type': 'Organization', 'name': 'Zexa', 'url': 'https://zexa.cc' },
        'publisher': { '@type': 'Organization', 'name': 'Zexa', 'url': 'https://zexa.cc' },
        'mainEntityOfPage': url,
        'image': `${SITE}/og-image.png`,
      }
      if (frontmatter.date) {
        article['datePublished'] = frontmatter.date
      }
      // dateModified 取 git 最后提交时间（lastUpdated: true 提供，ms 时间戳），真实的新鲜度信号
      const lastUpdated = context.pageData.lastUpdated
      if (lastUpdated) {
        article['dateModified'] = new Date(lastUpdated).toISOString()
      }
      head.push(['script', { type: 'application/ld+json' }, JSON.stringify(article)])
    }
    // 工具页不再输出 HowTo schema：Google 已于 2023-09 停用 HowTo 富结果，属无效投入

    return head
  },
})
