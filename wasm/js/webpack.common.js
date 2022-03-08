const path = require("path");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

module.exports = {
    entry: {
        app: { import: "./src/index.tsx", filename: "app.[contenthash:8].js" }
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
            filename: "index.html"
        }),
        new WasmPackPlugin({
            crateDirectory: "..",
            outDir: "./js/pkg"
        })
    ],
    output: {
        library: "jvm",
        filename: "[name].[contenthash:8].js",
        path: path.resolve(__dirname, "dist")
    },
    optimization: {
        splitChunks: {
            chunks: 'all',
        },
    },
}