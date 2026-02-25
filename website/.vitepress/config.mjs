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
                text: 'Custom Engine Features',
                items: [
                    { text: 'Long-Term Memory', link: '/features/memory' },
                    { text: 'Headless Web Search', link: '/features/search' },
                    { text: 'Azure Anthropic Support', link: '/features/azure-anthropic' },
                    { text: 'Automated Releases', link: '/features/releases' },
                    { text: 'LSP Symbol Search', link: '/features/lsp' }
                ]
            }
        ],

        socialLinks: [
            { icon: 'github', link: 'https://github.com/shotsan/zed-custom' }
        ],

        footer: {
            message: 'Forked with love from Zed Industries.',
            copyright: 'Copyright © 2026'
        },

        search: {
            provider: 'local'
        }
    }
})
