const path = require("path");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

module.exports = {
    entry: {
        wasmjvm: { import: "./src/index.ts", filename: "jvm.js" }
    },
    experiments: {
        asyncWebAssembly: true
    },
    module: {
        rules: [
            {
                test: /\.tsx?/,
                use: [{ loader: "ts-loader" }],
                exclude: /node_modules/
            }
        ]
    },
    resolve: {
        extensions: [".tsx", ".ts", ".js", ".wasm"],
        plugins: [
            new TsconfigPathsPlugin()
        ]
    },
    plugins: [
        new CleanWebpackPlugin(),
        new HtmlWebpackPlugin({
            template: "public/index.html",
            filename: "index.html",
            chunks: ["wasmjvm"]
        }),
        new WasmPackPlugin({
            crateDirectory: ".."
        })
    ],
    output: {
        filename: "[name].js",
        path: path.resolve(__dirname, "dist")
    },
};