/** @type {import('next').NextConfig} */
const nextConfig = {
  // output: 'export', // Disabled: API routes require server runtime
  distDir: 'dist',
  images: {
    unoptimized: true
  },
  trailingSlash: true
}

module.exports = nextConfig
