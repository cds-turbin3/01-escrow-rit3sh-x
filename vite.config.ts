import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"
import { nodePolyfills } from "vite-plugin-node-polyfills"
import { fileURLToPath } from "node:url"
import path from "node:path"

const root = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
    plugins: [
        react(),
        tailwindcss(),
        nodePolyfills({
            globals: { Buffer: true, process: true, global: true },
            protocolImports: true,
        }),
    ],
    resolve: {
        alias: {
            "@": path.resolve(root, "frontend"),
            "@contracts": path.resolve(root, "target/types"),
            "@idl": path.resolve(root, "target/idl"),
        },
    },
})
