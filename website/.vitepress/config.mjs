import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "Zed Custom",
    description: "An enhanced fork of the Zed code editor with native AI memory and headless search.",
    base: '/zed-custom/',

    head: [
        ['link', { rel: 'icon', type: 'image/png', href: '/zed-custom/favicon.png' }]
    ],

    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        logo: '/logo-animated.svg',

        nav: [
            { text: 'Home', link: '/' },
            { text: 'Features', link: '/features/memory' }
        ],

        sidebar: [
            {
                text: 'Core Features',
                items: [
                    { text: 'Agent Profiles', link: '/features/agent-profiles' },
                    { text: 'Skill Library', link: '/features/skill-library' },
                    { text: 'Long-Term Memory', link: '/features/memory' },
                    { text: 'Auto Context Compression', link: '/features/context-compression' },
                    { text: 'Headless Web Search', link: '/features/search' },
                    { text: 'Deep Research Tool', link: '/features/tools/deep-research' },
                    { text: 'Azure Anthropic Caching', link: '/features/azure-anthropic' },
                    { text: 'Prompt Caching Logic', link: '/features/prompt-caching' },
                    { text: 'LSP Symbol Search', link: '/features/lsp' },
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
                    { text: 'Web: deep_research', link: '/features/tools/deep-research' },
                    { text: 'LSP: context', link: '/features/tools/context' },
                    { text: 'Custom Search: custom_search + /custom-search', link: '/features/tools/custom-search' }
                ]
            },
            {
                text: 'Internals & Concepts',
                items: [
                    { text: 'Tree-sitter, AST, LSP & Indexing', link: '/features/internals' }
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
