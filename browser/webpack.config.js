const webpack = require('webpack')
const path = require('path')

const production = process.env.NODE_ENV === 'production'

// cli options, e.g. `--env.foo bar --env.baz` -> {foo: "bar", baz: true}
module.exports = (options = {}) => ({

  // The base directory for `entry` and `module`
  context: path.join(__dirname, '/src'),

  // The point or points to enter the application
  entry: (production ? [] : [
    // bundle the client for webpack dev server and connect to the provided endpoint
    'webpack-dev-server/client?http://localhost:8080',

    // bundle the client for hot reloading. only- means to only hot reload for successful updates
    'webpack/hot/only-dev-server'
  ]).concat([

    // the entry point of our app
    './index.js'
  ]),

  // where/how webpack should output bundled/loaded things
  output: {

    // output directory
    path: path.join(__dirname, '/dist'),

    // name of output bundle
    filename: 'bundle.js',

    // in the browser, the public URL of the output directory
    publicPath: '/'
  },

  // config for js, css, etc modules
  module: {

    // rules for building modules
    rules: [{

      // this rule is for all js files (under `context`)
      test: /\.js$/,

      // transpile w babel
      loader: 'babel-loader',

      options: {

        // babel presets
        presets: [

          // transpile es2015 code, without transpiling modules to commonjs
          // (since webpack2 uses native modules for tree shaking)
          ['es2015', { modules: false }],
          // for object spreads
          // 'stage-3'
        ]
      }
    }, {
      test: /\.vert|\.frag$/,
      loader: 'webpack-glsl-loader'
    }]
  },

  // what kind of source map to generate
  // > TL;DR For development, use cheap-module-eval-source-map. For production, use cheap-module-source-map.
  // http://cheng.logdown.com/posts/2016/03/25/679045
  devtool: production ? 'module-source-map' : 'module-eval-source-map',

  // webpack-dev-server config
  devServer: {

    // activates HMR
    hot: true,

    // Tell the server where to serve content from. This is only necessary if you want to serve static files.
    // (this is currently used to serve src/index.html)
    contentBase: path.join(__dirname, '/src')
  },

  plugins: [
    // activates HMR
    new webpack.HotModuleReplacementPlugin(),

    // prints more readable module names in the browser console on HMR updates
    new webpack.NamedModulesPlugin()
  ].concat(production ? [
    // `webpack -p` does the following:
    // - minifies
    // - define process.env.NODE_ENV="production"

    // reduce size of module/chunk IDs
    new webpack.optimize.OccurrenceOrderPlugin()
  ] : [])
})
