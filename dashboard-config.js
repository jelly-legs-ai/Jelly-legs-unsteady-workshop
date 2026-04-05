// Config injected at build/serve time
// IMPORTANT: Set GITHUB_TOKEN environment variable or configure this file before use
window.DASHBOARD_CONFIG = {
    // Set your GitHub Personal Access Token here or via GITHUB_TOKEN env var
    GITHUB_TOKEN: window.localStorage.getItem('github_token') || '',
    GITHUB_REPO: 'jelly-legs-ai/Jelly-legs-unsteady-workshop',
    API_BASE_URL: 'https://api.github.com'
};

// Make token available globally for backward compatibility
window.GITHUB_TOKEN = window.DASHBOARD_CONFIG.GITHUB_TOKEN;
