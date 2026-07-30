import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // 第3引数 '' でプレフィックス制限を外し、VITE_ なしの変数も読む。
  // DEV_API_PROXY_TARGET は開発サーバー専用の設定であり、バンドルには一切埋め込まれない
  // （import.meta.env 経由で参照しないため）。
  const env = loadEnv(mode, __dirname, '')

  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },
    // src/lib/apiClient.ts は API を相対パス /api/v1 で呼ぶ（本番は nginx.conf が
    // /api/ を backend へ proxy する）。開発サーバーでも同じ相対パスが通るよう転送する。
    server: {
      proxy: {
        '/api': {
          target: env.DEV_API_PROXY_TARGET || 'http://localhost:8080',
          changeOrigin: true,
        },
      },
    },
  }
})
