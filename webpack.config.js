const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyPlugin = require("copy-webpack-plugin");
const webpack = require("webpack");

const dist = path.resolve(__dirname, "dist");
const pkg = path.resolve(__dirname, "pkg");

module.exports = {
  mode: "production",
  module: {
    rules: [{
      test: /\.wasm$/,
      type: "webassembly/async"
    }]
  },
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
    index: "./static/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    static: [dist, pkg],
  },
  performance: { hints: false },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: '--features=web'
    }),
    new CopyPlugin({
      patterns: [
        path.resolve(__dirname, "static")
      ]
    }),

    new webpack.LoaderOptionsPlugin({
      options: {
        experiments: {
          asyncWebAssembly: true
        }
      }
    }),
  ]
};
