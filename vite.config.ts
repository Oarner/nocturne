import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    watch: {
      ignored: ['**/AppData/**', '**/*.vol', '**/*.jfm', '**/src-tauri/target/**']
    }
  }
})