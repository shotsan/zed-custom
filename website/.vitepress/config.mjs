import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "Zed Custom",
    description: "An enhanced fork of the Zed code editor with native AI memory and headless search.",
    base: '/zed-custom/',
    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        logo: 'https://zed.dev/zed-logo.svg', // Placeholder, using Zed logo for now

        nav: [
            { text: 'Home', link: '/' },
            { text: 'Features', link: '/features/memory' }
        ],

        sidebar: [
            {
                text: 'Core Features',
                items: [
                    { text: 'Long-Term Memory', link: '/features/memory' },
                    { text: 'Headless Web Search', link: '/features/search' },
                    { text: 'Azure Anthropic Caching', link: '/features/azure-anthropic' },
                    { text: 'LSP Symbol Search', link: '/features/lsp' },
                    { text: 'Custom Rules (.rules)', link: '/features/rules' },
                    { text: 'System Prompts & Persona', link: '/features/system-prompts' },
                    { text: 'Full Message Interception', link: '/features/message-interception' }
                ]
            },
            {
                text: 'Agent Tools',
                items: [
                    { text: 'Memory: remember', link: '/features/tools/remember' },
                    { text: 'Memory: recall', link: '/features/tools/recall' },
                    { text: 'Web: search', link: '/features/tools/search' },
                    { text: 'Web: fetch', link: '/features/tools/fetch' },
                    { text: 'LSP: context', link: '/features/tools/context' }
                ]
            }
        ],

        socialLinks: [
            { icon: 'github', link: 'https://github.com/shotsan/zed-custom' }
        ],

        footer: {
            message: 'Unofficial Zed Fork. Not affiliated with Zed Industries.',
            copyright: 'Copyright © 2026 shotsan. Modifications under MIT License.'
        },

        search: {
            provider: 'local'
        }
    }
})
