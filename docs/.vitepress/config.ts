import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'hudo',
  description: 'Windows 开发环境一键引导工具',
  lang: 'zh-CN',

  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
  ],

  themeConfig: {
    logo: '/logo.png',
    siteTitle: 'hudo',

    nav: [
      { text: '指南', link: '/guide/what-is-hudo' },
      { text: '工具列表', link: '/tools/' },
      {
        text: 'v0.1.5',
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
            { text: 'Bun', link: '/tools/bun' },
            { text: 'Rust', link: '/tools/rust' },
            { text: 'Go', link: '/tools/go' },
            { text: 'JDK', link: '/tools/jdk' },
            { text: 'Maven', link: '/tools/maven' },
            { text: 'Gradle', link: '/tools/gradle' },
            { text: 'Python (uv)', link: '/tools/python' },
            { text: 'Miniconda', link: '/tools/miniconda' },
            { text: 'MySQL', link: '/tools/mysql' },
            { text: 'PostgreSQL', link: '/tools/pgsql' },
            { text: 'VS Code', link: '/tools/vscode' },
            { text: 'PyCharm', link: '/tools/pycharm' },
            { text: 'MinGW', link: '/tools/mingw' },
            { text: 'Google Chrome', link: '/tools/chrome' },
            { text: 'Claude Code', link: '/tools/claude-code' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/zexadev/hudo' }
    ],

    footer: {
      message: '基于 MIT 协议发布',
      copyright: 'Copyright © 2025 Zexa'
    },

    search: {
      provider: 'local'
    },

    editLink: {
      pattern: 'https://github.com/zexadev/hudo/edit/master/docs/:path',
      text: '在 GitHub 上编辑此页'
    }
  }
})
